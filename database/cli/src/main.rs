// database status
// create admin
// change admin password
// start database
// stop database
// restart database
// update database
#![allow(dead_code)]

use clap::{arg, command, value_parser};
use reqwest::StatusCode;

use std::io::Write;
use std::time::Duration;

fn main() {
    let client = reqwest::blocking::Client::new();
    let addr = database_common::ADDR.to_string();

    let server = Server { client, addr };

    let cmd = derive_server_command();
    dbg!(&cmd);

    server.handle(cmd);
}

fn derive_server_command() -> ServerCommands {
    let matches = command!() // requires `cargo` feature
        .arg(
            arg!(args: [COMMANDS])
                .num_args(1..)
                .value_parser(value_parser!(String)),
        )
        .get_matches();

    let q = matches
        .get_many::<String>("args")
        .map(|vals| vals.collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str())
        .collect::<Vec<&str>>();

    ServerCommands::try_from(q).unwrap()
}

fn read_input(before: impl AsRef<str>) -> String {
    let _ = std::io::stdout().write(before.as_ref().as_bytes()).unwrap();
    let _: () = std::io::stdout().flush().unwrap();
    let mut input_string = String::new();
    std::io::stdin().read_line(&mut input_string).unwrap();
    input_string.strip_suffix("\n").unwrap().to_owned()
}

fn read_secret(before: impl AsRef<str>) -> String {
    rpassword::prompt_password(before.as_ref()).unwrap()
}

struct Server {
    client: reqwest::blocking::Client,
    addr: String,
}

impl Server {
    fn status(&self) -> Status {
        let r = self
            .client
            .get(format!("http://{}/health", self.addr))
            .timeout(Duration::from_secs(1))
            .send();

        match r {
            Ok(r) => match r.status() {
                StatusCode::OK => Status::Alive,
                _ => unreachable!(),
            },
            Err(_e) => Status::Dead,
        }
    }

    fn database_info(&self) -> interfacing::DatabaseInfo {
        let r = self
            .client
            .get(format!("http://{}/database/info", self.addr))
            .timeout(Duration::from_secs(1))
            .send()
            .unwrap()
            .json::<interfacing::DatabaseInfo>()
            .unwrap();
        r
    }
}

#[derive(Debug)]
enum Status {
    Alive,
    Dead,
}

#[derive(Debug)]
enum ServerCommands {
    HTTPServerStatus,
    DatabaseInfo,
}

impl TryFrom<Vec<&str>> for ServerCommands {
    type Error = String;
    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        use ServerCommands as Cmd;
        match value[..] {
            ["http_server", "status"] => Ok(Cmd::HTTPServerStatus),
            ["db", "info"] => Ok(Cmd::DatabaseInfo),
            _ => Err("invalid command".into()),
        }
    }
}

impl Server {
    fn handle(&self, cmd: ServerCommands) {
        use ServerCommands as Cmd;
        match cmd {
            Cmd::HTTPServerStatus => {
                let status = self.status();
                dbg!(status);
            }
            Cmd::DatabaseInfo => {
                let info = self.database_info();
                dbg!(info);
            }
        }
    }
}

// std::process::Command::new("cargo")
//     .arg("run")
//     .current_dir("../http_server")
//     .process_group(0)
//     .spawn()
//     .expect("failed to execute process");
