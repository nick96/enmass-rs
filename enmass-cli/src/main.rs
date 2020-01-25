use dotenv;
use engine;
use std::default::Default;
use structopt::clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(PartialEq, Debug, Clone)]
    enum ContactType {
        Email,
        Phone,
    }
}

arg_enum! {
    #[derive(PartialEq, Debug, Clone)]
    enum Command {
        Get,
        Send,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(version = "0.0.1", author = "Nick Spain")]
struct Opts {
    #[structopt(long = "auth-uri", env = "AUTH_URI")]
    auth_uri: String,
    #[structopt(long = "token-uri", env = "TOKEN_URI")]
    token_uri: String,
    #[structopt(long = "redirect-uris", env = "REDIRECT_URIS")]
    redirect_uris: Vec<String>,
    #[structopt(long = "client-id", env = "CLIENT_ID")]
    client_id: String,
    #[structopt(long = "secret", env = "CLIENT_SECRET")]
    client_secret: String, // TODO: Look into ways to make this unloggable (i.e. redacted)
    #[structopt()]
    command: Command,
    #[structopt()]
    contact_type: ContactType,
    #[structopt()]
    group_name: String,
}

fn main() {
    dotenv::dotenv().ok();

    let opts = Opts::from_args();
    let secret = engine::ApplicationSecret {
        auth_uri: opts.auth_uri,
        token_uri: opts.token_uri,
        redirect_uris: opts.redirect_uris,
        client_id: opts.client_id,
        client_secret: opts.client_secret,
        ..Default::default()
    };

    let group_name = &opts.group_name;

    let engine = {
        let authenticator =
            engine::authenticator(&secret, engine::hyper_client(), "token.json".to_string());
        let client = engine::hyper_client();
        engine::PeopleEngine::new(client, authenticator)
    };

    let group_members: Vec<engine::Person> = match engine.get_members(group_name) {
        Ok(group_members) => group_members,
        Err(e) => panic!(format!(
            "Could not get members of group {}: {:?}",
            group_name, e
        )),
    };
    match opts.command {
        Command::Get => {
            let details: Vec<String> = match opts.contact_type {
                ContactType::Email => group_members
                    .iter()
                    .map(|member| {
                        member
                            .clone()
                            .email_addresses
                            .unwrap_or(Vec::default())
                            .iter()
                            .map(|email_addr| {
                                String::from(
                                    email_addr
                                        .value
                                        .clone()
                                        .unwrap_or(String::from("<missing>"))
                                        .trim(),
                                )
                            })
                            .collect()
                    })
                    .collect(),
                ContactType::Phone => group_members
                    .iter()
                    .map(|member| {
                        member
                            .clone()
                            .phone_numbers
                            .unwrap_or(Vec::default())
                            .iter()
                            .map(|phone| {
                                String::from(
                                    phone
                                        .value
                                        .clone()
                                        .unwrap_or(String::from("<missing>"))
                                        .trim(),
                                )
                            })
                            .collect()
                    })
                    .collect(),
            };
            println!("{}", details.join(";"));
        }
        Command::Send => unimplemented!(),
    }
}
