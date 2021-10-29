use crate::{new_rpc_client, Command, Error, Result};
use clap::value_t_or_exit;
use itertools::Itertools;
use mullvad_management_interface::{
    types::{self, Timestamp},
    Code,
};
use mullvad_types::account::AccountToken;
use std::io::{self, Write};

pub struct Account;

#[mullvad_management_interface::async_trait]
impl Command for Account {
    fn name(&self) -> &'static str {
        "account"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Control and display information about your Mullvad account")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("create").about("Create and log in to a new account"),
            )
            .subcommand(
                clap::SubCommand::with_name("login")
                    .about("Log in to an account")
                    .arg(
                        clap::Arg::with_name("token")
                            .help("The Mullvad account token to configure the client with")
                            .required(false),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("logout").about("Log out of the current account"),
            )
            .subcommand(
                clap::SubCommand::with_name("get")
                    .about("Display information about the current account"),
            )
            .subcommand(
                clap::SubCommand::with_name("list-devices")
                    .about("List devices associated with an account")
                    .arg(
                        clap::Arg::with_name("token")
                            .help("Mullvad account number")
                            .required(true),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("revoke-device")
                    .about("Revoke a device associated with an account")
                    .arg(
                        clap::Arg::with_name("token")
                            .help("Mullvad account number")
                            .required(true),
                    )
                    .arg(
                        clap::Arg::with_name("device")
                            .help("ID of the device to revoke")
                            .required(true),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("redeem")
                    .about("Redeems a voucher")
                    .arg(
                        clap::Arg::with_name("voucher")
                            .help("The Mullvad voucher code to be submitted")
                            .required(true),
                    ),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(_matches) = matches.subcommand_matches("create") {
            self.create().await
        } else if let Some(set_matches) = matches.subcommand_matches("login") {
            self.login(parse_token(set_matches)).await
        } else if let Some(_matches) = matches.subcommand_matches("logout") {
            self.logout().await
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get().await
        } else if let Some(set_matches) = matches.subcommand_matches("list-devices") {
            self.list_devices(parse_token(set_matches)).await
        } else if let Some(set_matches) = matches.subcommand_matches("revoke-device") {
            self.revoke_device(parse_token(set_matches), parse_device_id(set_matches))
                .await
        } else if let Some(matches) = matches.subcommand_matches("redeem") {
            let voucher = value_t_or_exit!(matches.value_of("voucher"), String);
            self.redeem_voucher(voucher).await
        } else {
            unreachable!("No account command given");
        }
    }
}

impl Account {
    async fn create(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.create_new_account(()).await?;
        println!("New account created!");
        self.get().await
    }

    async fn login(&self, token: AccountToken) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.login_account(token.clone()).await?;
        println!("Mullvad account \"{}\" set", token);
        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.logout_account(()).await?;
        println!("Removed device from Mullvad account");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let device = rpc.get_device(()).await?.into_inner();
        if !device.account_token.is_empty() {
            println!("Mullvad account: {}", device.account_token);
            println!("Device name    : {}", device.device.unwrap().name);
            let expiry = rpc
                .get_account_data(device.account_token)
                .await
                .map_err(|error| Error::RpcFailedExt("Failed to fetch account data", error))?
                .into_inner();
            println!(
                "Expires at     : {}",
                Self::format_expiry(&expiry.expiry.unwrap())
            );
        } else {
            println!("No account configured");
        }
        Ok(())
    }

    async fn list_devices(&self, token: String) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let devices = rpc.list_devices(token).await?.into_inner();

        println!("{:?}", devices);

        Ok(())
    }

    async fn revoke_device(&self, token: String, device_id: String) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.remove_device(types::DeviceRemoval {
            account_token: token,
            device_id,
        })
        .await?;
        println!("Removed device");
        Ok(())
    }

    async fn redeem_voucher(&self, mut voucher: String) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        voucher.retain(|c| c.is_alphanumeric());

        match rpc.submit_voucher(voucher).await {
            Ok(submission) => {
                let submission = submission.into_inner();
                println!(
                    "Added {} to the account",
                    Self::format_duration(submission.seconds_added)
                );
                println!(
                    "New expiry date: {}",
                    Self::format_expiry(&submission.new_expiry.unwrap())
                );
                Ok(())
            }
            Err(err) => {
                match err.code() {
                    Code::NotFound | Code::ResourceExhausted => {
                        eprintln!("Failed to submit voucher: {}", err.message());
                    }
                    _ => return Err(Error::RpcFailed(err)),
                }
                std::process::exit(1);
            }
        }
    }

    fn format_duration(seconds: u64) -> String {
        let dur = chrono::Duration::seconds(seconds as i64);
        if dur.num_days() > 0 {
            format!("{} days", dur.num_days())
        } else if dur.num_hours() > 0 {
            format!("{} hours", dur.num_hours())
        } else if dur.num_minutes() > 0 {
            format!("{} minutes", dur.num_minutes())
        } else {
            format!("{} seconds", dur.num_seconds())
        }
    }

    fn format_expiry(expiry: &Timestamp) -> String {
        let ndt = chrono::NaiveDateTime::from_timestamp(expiry.seconds, expiry.nanos as u32);
        let utc = chrono::DateTime::<chrono::Utc>::from_utc(ndt, chrono::Utc);
        utc.with_timezone(&chrono::Local).to_string()
    }
}

fn parse_token(matches: &clap::ArgMatches<'_>) -> String {
    parse_from_match_else_stdin("Enter account token: ", "token", matches)
}

fn parse_device_id(matches: &clap::ArgMatches<'_>) -> String {
    parse_from_match_else_stdin("Enter device id: ", "device", matches)
}

fn parse_from_match_else_stdin(
    prompt_str: &'static str,
    key: &'static str,
    matches: &clap::ArgMatches<'_>,
) -> String {
    let val = match matches.value_of(key) {
        Some(device) => device.to_string(),
        None => {
            let mut val = String::new();
            io::stdout()
                .write_all(prompt_str.as_bytes())
                .expect("Failed to write to STDOUT");
            let _ = io::stdout().flush();
            io::stdin()
                .read_line(&mut val)
                .expect("Failed to read from STDIN");
            val
        }
    };
    val.split_whitespace().join("").to_string()
}
