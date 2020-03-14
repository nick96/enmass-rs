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
    enum SendMethod {
        Text, Email
    }
}

#[derive(StructOpt, Debug)]
enum Command {
  #[structopt(about = "Get email/phone contact info enmass")]
  Get {
    #[structopt(about = "contact type to get")]
    contact_type: ContactType,
    #[structopt(about = "name of the group to get the contact info from")]
    group_name: String,
  },
  #[structopt(about = "Send emails/text enmass")]
  Send {
    #[structopt(about = "method to send the message via")]
    method: SendMethod,
    #[structopt(about = "name of the group to send the message to")]
    group_name: String,
  },
}

#[derive(StructOpt, Debug)]
#[structopt(
  version = "0.0.1",
  author = "Nick Spain",
  about = "Send emails/texts and get contact info enmass"
)]
struct Opts {
  // Config items for authentication
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

  #[structopt(subcommand)]
  command: Command,
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

  let engine = {
    let authenticator =
      engine::authenticator(&secret, engine::hyper_client(), "token.json".to_string());
    let client = engine::hyper_client();
    engine::PeopleEngine::new(client, authenticator)
  };

  match opts.command {
    Command::Get {
      contact_type,
      group_name,
    } => {
      let details = match contact_type {
        ContactType::Email => engine.get_group_emails(&group_name),
        ContactType::Phone => engine.get_group_phones(&group_name),
      };
      match details {
        Ok(details) => println!("{}", details.join(";")),
        Err(e) => write_error(e),
      }
    }
    Command::Send { method, group_name } => match method {
      SendMethod::Email => {
        let _emails = engine.get_group_emails(&group_name);
        todo!()
      }
      SendMethod::Text => {
        let _phones = engine.get_group_phones(&group_name);
        todo!()
      }
    },
  }
}
