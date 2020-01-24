use google_people1 as people;
use google_people1::PeopleService;
use hyper;
use hyper_rustls;
use snafu::Snafu;
use yup_oauth2 as oauth2;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not get contact groups: {}", source))]
    GetContactGroups { source: people::Error },

    #[snafu(display("Could not get contact group '{}': {}", group_name, source))]
    GetContactGroup {
        group_name: String,
        source: people::Error,
    },

    #[snafu(display("No resource name for contact group '{}' exists", group_name,))]
    GetContactGroupResourceName { group_name: String },

    #[snafu(display("No contact groups exist"))]
    NoContactGroups,

    #[snafu(display(
        "Found {} contact groups with the name '{}', there can only be one",
        found,
        group_name
    ))]
    NonUniqueContactGroupName { group_name: String, found: usize },

    #[snafu(display("No groups were found with the name '{}'", group_name))]
    NoContactGroupsFoundByName { group_name: String, message: String },

    #[snafu(display("No members were found in the group '{}'", group_name))]
    NoGroupMemberResourceNames { group_name: String },

    #[snafu(display(
        "Could not get person with resource name '{}': {}",
        resource_name,
        source
    ))]
    GetPersonByResourceName {
        resource_name: String,
        source: people::Error,
    },

    #[snafu(display("Could not get group members for group '{}'", group_name,))]
    GetContactGroupMembers {
        group_name: String,
        prev_err_msg: String,
    },
}

pub type Authenticator = oauth2::Authenticator<
    oauth2::DefaultAuthenticatorDelegate,
    oauth2::MemoryStorage,
    hyper::Client,
>;

pub type Service = PeopleService<hyper::Client, Authenticator>;

pub struct PeopleEngine {
    hub: Service,
}

impl PeopleEngine {
    pub fn new(client: hyper::Client, authenticator: Authenticator) -> Self {
        let hub = PeopleService::new(client, authenticator);
        PeopleEngine { hub: hub }
    }

    pub fn get_contact_groups(&self) -> Result<Vec<people::ContactGroup>, Error> {
        match self.hub.contact_groups().list().doit() {
            Ok((_, contact_groups_resp)) => match contact_groups_resp.contact_groups {
                Some(contact_groups) => Ok(contact_groups),
                None => Err(Error::NoContactGroups),
            },
            Err(e) => Err(Error::GetContactGroups { source: e }),
        }
    }

    pub fn get_contact_group(&self, group_name: &String) -> Result<people::ContactGroup, Error> {
        let contact_group_without_members = match self.get_contact_groups() {
            Ok(contact_groups) => {
                let selected_groups: Vec<&people::ContactGroup> = contact_groups
                    .iter()
                    .filter(|cg| cg.name.clone().unwrap_or("".to_string()) == group_name.clone())
                    .collect();
                if selected_groups.is_empty() {
                    let mut group_names: Vec<String> = contact_groups
                        .iter()
                        .map(|cg| cg.name.clone().unwrap().clone())
                        .collect();
                    group_names.sort_by(|cg_name1, cg_name2| {
                        strsim::levenshtein(cg_name1, group_name)
                            .partial_cmp(&strsim::levenshtein(cg_name2, group_name))
                            .unwrap()
                    });

                    if let Some(closest) = group_names.get(0) {
                        Err(Error::NoContactGroupsFoundByName {
                            group_name: group_name.to_string(),
                            message: format!("Did you mean {}", closest),
                        })
                    } else {
                        Err(Error::NoContactGroupsFoundByName {
                            group_name: group_name.to_string(),
                            message: String::from(""),
                        })
                    }
                } else if selected_groups.len() != 1 {
                    Err(Error::NonUniqueContactGroupName {
                        group_name: group_name.to_string(),
                        found: selected_groups.len(),
                    })
                } else {
                    Ok(selected_groups.first().unwrap().clone().clone())
                }
            }
            Err(e) => Err(e),
        };

        match contact_group_without_members {
            Ok(contact_group_without_members) => {
                match contact_group_without_members.resource_name {
                    Some(resource_name) => {
                        let result = self
                            .hub
                            .contact_groups()
                            .get(&resource_name)
                            .max_members(100)
                            .doit();
                        match result {
                            Ok((_, contact_group)) => Ok(contact_group),
                            Err(e) => Err(Error::GetContactGroup {
                                group_name: group_name.to_string(),
                                source: e,
                            }),
                        }
                    }
                    None => Err(Error::GetContactGroupResourceName {
                        group_name: group_name.to_string(),
                    }),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_member_by_resource_name(
        &self,
        resource_name: &String,
    ) -> Result<people::Person, Error> {
        match self
            .hub
            .people()
            .get(resource_name)
            .person_fields("emailAddresses,phoneNumbers")
            .doit()
        {
            Ok((_, person)) => Ok(person),
            Err(e) => Err(Error::GetPersonByResourceName {
                resource_name: resource_name.to_string(),
                source: e,
            }),
        }
    }

    pub fn get_members(&self, group_name: &String) -> Result<Vec<people::Person>, Error> {
        match self.get_contact_group(group_name) {
            Ok(group) => match group.member_resource_names {
                Some(resource_names) => {
                    let members: Vec<Result<people::Person, Error>> = resource_names
                        .iter()
                        .map(|rn| match self.get_member_by_resource_name(rn) {
                            Ok(member) => Ok(member),
                            Err(e) => Err(e),
                        })
                        .collect();
                    for member in &members {
                        if let Err(e) = member {
                            return Err(Error::GetContactGroupMembers {
                                group_name: group_name.to_string(),
                                prev_err_msg: format!("{}", e),
                            });
                        }
                    }
                    Ok(members
                        .iter()
                        .map(|member| member.as_ref().unwrap().clone())
                        .collect())
                }
                None => Err(Error::NoGroupMemberResourceNames {
                    group_name: group_name.to_string(),
                }),
            },
            Err(e) => Err(e),
        }
    }
}

pub fn hyper_client() -> hyper::Client {
    let https_connector = hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new());
    hyper::Client::with_connector(https_connector)
}

pub fn authenticator(secret: &oauth2::ApplicationSecret, client: hyper::Client) -> Authenticator {
    oauth2::Authenticator::new(
        secret,
        oauth2::DefaultAuthenticatorDelegate,
        client,
        <oauth2::MemoryStorage as Default>::default(),
        Some(oauth2::FlowType::InstalledRedirect(54324)),
    )
}
