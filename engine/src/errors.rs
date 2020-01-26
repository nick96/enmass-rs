use snafu;
use snafu::{Backtrace, Snafu};

use google_people1 as people;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not get contact groups: {}", source))]
    GetContactGroups { source: people::Error },

    #[snafu(display("Could not get contact group '{}': {}", group_name, source))]
    GetContactGroup {
        group_name: String,
        source: people::Error,
    },

    #[snafu(display("No resource name for contact group '{}' exists", group_name,))]
    GetContactGroupResourceName {
        group_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("No contact groups exist"))]
    NoContactGroups { backtrace: Backtrace },

    #[snafu(display(
        "Found {} contact groups with the name '{}', there can only be one",
        found,
        group_name
    ))]
    NonUniqueContactGroupName {
        group_name: String,
        found: usize,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "No groups were found with the name '{}', did you mean '{}'?",
        group_name,
        closest
    ))]
    NoContactGroupsFoundByName { group_name: String, closest: String },

    #[snafu(display("No members were found in the group '{}'", group_name))]
    NoGroupMemberResourceNames {
        group_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "Could not get person with resource name '{}': {}",
        resource_name,
        source
    ))]
    GetPersonByResourceName {
        resource_name: String,
        source: people::Error,
    },

    #[snafu(display("Could not get group members for group '{}': {}", group_name, source))]
    GetContactGroupMembers {
        group_name: String,
        source: people::Error,
    },

    #[snafu(display("Could not get emails for group '{}': {}", group_name, source))]
    GetGroupEmails {
        group_name: String,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
    },

    #[snafu(display("Could not get phones for group '{}': {}", group_name, source))]
    GetGroupPhones {
        group_name: String,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
    },
    #[snafu(display("Found a 'None' person in group '{}'", group_name))]
    NonePersonInGroup {
        group_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Could not get members in group '{}': {}", group_name, source))]
    GetMembers {
        group_name: String,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
    },

    #[snafu(display(
        "No contact group members were found for contact group '{}'",
        group_name,
    ))]
    MissingContactGroupMembers {
        group_name: String,
        backtrace: Backtrace,
    },
}
