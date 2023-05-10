// database status +
// create dashboard admin +
// change dashboard admin password +
// create database admin
// change database admin password +
// start database
// stop database
// restart database
// update database
// plain backup database +
#![allow(dead_code)]

use clap::{arg, command, value_parser};
use reqwest::StatusCode;

use std::io::Write;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();
    let addr = database_common::ADDR.to_string();

    let server = Server { client, addr };

    let cmd = derive_server_command();
    server.handle(cmd)
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

    ServerCommands::from(q)
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
    fn replace_dashboard_admin(&self) -> anyhow::Result<()> {
        let password = read_input("Enter new password:");
        let r = self
            .client
            .post(format!("http://{}/users/", self.addr))
            .timeout(Duration::from_secs(1))
            .json(&interfacing::LoginForm {
                username: "admin".into(),
                password: secrecy::SecretString::from(password),
            })
            .send()?;

        match r.status() {
            StatusCode::OK => println!("{}", r.text()?),
            _ => println!("failed to create user"),
        }
        Ok(())
    }

    fn update_database_admin_password(&self) -> anyhow::Result<()> {
        let password = read_input("Enter new password:");
        let r = self
            .client
            .post(format!("http://{}/database/users/", self.addr))
            .timeout(Duration::from_secs(3))
            .json(&interfacing::LoginForm {
                username: "admin".into(),
                password: secrecy::SecretString::from(password),
            })
            .send()?;

        match r.status() {
            StatusCode::OK => println!("{}", r.text()?),
            _ => println!("failed to create user"),
        }

        Ok(())
    }

    fn backup_database(&self) {
        let r = self
            .client
            .get(format!("http://{}/database/backup", self.addr))
            .timeout(Duration::from_secs(3))
            .send();

        match r {
            Ok(r) => match r.status() {
                StatusCode::OK => println!("database has been backed up"),
                _ => unimplemented!(),
            },
            Err(_e) => unimplemented!(),
        }
    }

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
    DashboardAdminReplace,
    DatabaseAdminPassword,
    DatabaseBackupCreate,
    InvalidCommand,
}

impl From<Vec<&str>> for ServerCommands {
    fn from(value: Vec<&str>) -> Self {
        use ServerCommands as Cmd;
        let cmd = match value[..] {
            ["http_server", "status"] => Cmd::HTTPServerStatus,
            ["db", "info"] => Cmd::DatabaseInfo,
            ["db", "admin", "password"] => Cmd::DatabaseAdminPassword,
            ["db", "backup", "create"] => Cmd::DatabaseBackupCreate,
            ["dashboard", "admin", "replace"] => Cmd::DashboardAdminReplace,
            _ => Cmd::InvalidCommand,
        };
        cmd
    }
}

impl Server {
    fn handle(&self, cmd: ServerCommands) -> anyhow::Result<()> {
        use ServerCommands as Cmd;
        match cmd {
            Cmd::InvalidCommand => {
                println!("invalid command");
            }
            Cmd::HTTPServerStatus => {
                let status = self.status();
                println!("{:?}", status);
            }
            Cmd::DatabaseInfo => {
                let info = self.database_info();
                println!("{:?}", info);
            }
            Cmd::DashboardAdminReplace => self.replace_dashboard_admin()?,
            Cmd::DatabaseAdminPassword => self.update_database_admin_password()?,
            Cmd::DatabaseBackupCreate => self.backup_database(),
        }
        Ok(())
    }
}

// std::process::Command::new("cargo")
//     .arg("run")
//     .current_dir("../http_server")
//     .process_group(0)
//     .spawn()
//     .expect("failed to execute process");
