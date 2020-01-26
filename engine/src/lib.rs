use google_people1 as people;
use hyper;
use hyper_rustls;
use snafu::{ensure, OptionExt, ResultExt};
use strsim;
use yup_oauth2 as oauth2;

pub mod errors;

pub use errors::Error;
pub use google_people1::Person;
pub use yup_oauth2::ApplicationSecret;

pub type Authenticator = oauth2::Authenticator<
    oauth2::DefaultAuthenticatorDelegate,
    oauth2::DiskTokenStorage,
    hyper::Client,
>;

pub type Service = people::PeopleService<hyper::Client, Authenticator>;

pub struct PeopleEngine {
    hub: Service,
}

type Result<T, E = errors::Error> = std::result::Result<T, E>;

impl PeopleEngine {
    pub fn new(client: hyper::Client, authenticator: Authenticator) -> Self {
        let hub = people::PeopleService::new(client, authenticator);
        PeopleEngine { hub: hub }
    }

    pub fn get_contact_groups(&self) -> Result<Vec<people::ContactGroup>> {
        let (_, contact_groups_resp) = self
            .hub
            .contact_groups()
            .list()
            .doit()
            .context(errors::GetContactGroups)?;
        ensure!(
            contact_groups_resp.contact_groups.is_some(),
            errors::NoContactGroups
        );
        Ok(contact_groups_resp.contact_groups.unwrap())
    }

    pub fn get_contact_group(&self, group_name: &String) -> Result<people::ContactGroup> {
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
                    let closest = group_names.get(0).context(errors::NoContactGroups)?;
                    errors::NoContactGroupsFoundByName {
                        group_name: group_name.to_string(),
                        closest: closest,
                    }
                    .fail()
                } else if selected_groups.len() != 1 {
                    errors::NonUniqueContactGroupName {
                        group_name: group_name.to_string(),
                        found: selected_groups.len(),
                    }
                    .fail()
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
                        let (_, contact_group) = self
                            .hub
                            .contact_groups()
                            .get(&resource_name)
                            .max_members(100)
                            .doit()
                            .context(errors::GetContactGroup {
                                group_name: group_name.to_string(),
                            })?;
                        Ok(contact_group)
                    }
                    None => errors::GetContactGroupResourceName {
                        group_name: group_name.to_string(),
                    }
                    .fail(),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_member_by_resource_name(&self, resource_name: &String) -> Result<people::Person> {
        let (_, member) = self
            .hub
            .people()
            .get(resource_name)
            .person_fields("emailAddresses,phoneNumbers")
            .doit()
            .context(errors::GetPersonByResourceName {
                resource_name: resource_name.to_string(),
            })?;
        Ok(member)
    }

    pub fn get_members(&self, group_name: &String) -> Result<Vec<people::Person>> {
        let group = self
            .get_contact_group(group_name)
            .context(errors::GetMembers {
                group_name: group_name.to_string(),
            })?;
        let resource_names =
            group
                .member_resource_names
                .context(errors::NoGroupMemberResourceNames {
                    group_name: group_name.to_string(),
                })?;
        let mut request = self.hub.people().get_batch_get();
        for rn in resource_names {
            request = request.add_resource_names(&rn);
        }

        let (_, response) = request
            .person_fields("emailAddresses,phoneNumbers")
            .doit()
            .context(errors::GetContactGroupMembers {
                group_name: group_name.to_string(),
            })?;

        let people = response
            .responses
            .context(errors::MissingContactGroupMembers {
                group_name: group_name.to_string(),
            })?;
        if people.iter().any(|p| p.person.is_none()) {
            return errors::NonePersonInGroup {
                group_name: group_name.to_string(),
            }
            .fail();
        }
        let members = people
            .iter()
            .map(|p| p.person.as_ref().unwrap().clone())
            .collect();
        Ok(members)
    }

    pub fn get_group_emails(&self, group_name: &String) -> Result<Vec<String>> {
        let group_members = self
            .get_members(group_name)
            .context(errors::GetGroupEmails {
                group_name: group_name.to_string(),
            })?;
        let emails: Vec<String> = group_members
            .iter()
            .flat_map(|member| {
                let email_addresses = member.email_addresses.as_ref().unwrap();
                email_addresses
                    .iter()
                    .map(|email_addr| email_addr.value.as_ref().unwrap().trim().to_string())
            })
            .collect::<Vec<String>>();
        Ok(emails)
    }

    pub fn get_group_phones(&self, group_name: &String) -> Result<Vec<String>> {
        let group_members = self
            .get_members(group_name)
            .context(errors::GetGroupPhones {
                group_name: group_name.to_string(),
            })?;
        let phones = group_members
            .iter()
            .map(|member| {
                member
                    .clone()
                    .phone_numbers
                    .unwrap_or(Vec::default())
                    .iter()
                    .map(|phone| String::from(phone.value.clone().unwrap().trim()))
                    .collect()
            })
            .collect();
        Ok(phones)
    }
}

pub fn hyper_client() -> hyper::Client {
    let https_connector = hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new());
    hyper::Client::with_connector(https_connector)
}

pub fn authenticator(
    secret: &oauth2::ApplicationSecret,
    client: hyper::Client,
    token_path: String,
) -> Authenticator {
    oauth2::Authenticator::new(
        secret,
        oauth2::DefaultAuthenticatorDelegate,
        client,
        oauth2::DiskTokenStorage::new(&token_path).unwrap(),
        Some(oauth2::FlowType::InstalledInteractive),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
