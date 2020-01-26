use dotenv;
use engine;
use snafu::ErrorCompat;
use std::default::Default;
use std::io::Write;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

fn write_error(error: engine::Error) {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    stderr
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
        .unwrap();
    write!(&mut stderr, "Failed: ").unwrap();
    stderr.reset().unwrap();
    writeln!(&mut stderr, "{}", error).unwrap();
    if let Some(backtrace) = error.backtrace() {
        eprintln!("{}", backtrace)
    }
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

    match opts.command {
        Command::Get => {
            let details = match opts.contact_type {
                ContactType::Email => engine.get_group_emails(group_name),
                ContactType::Phone => engine.get_group_phones(group_name),
            };
            match details {
                Ok(details) => println!("{}", details.join(";")),
                Err(e) => write_error(e),
            }
        }
        Command::Send => match opts.contact_type {
            ContactType::Email => {
                let _emails = engine.get_group_emails(group_name);
                todo!()
            }
            ContactType::Phone => {
                let _phones = engine.get_group_phones(group_name);
                todo!()
            }
        },
    }
}
