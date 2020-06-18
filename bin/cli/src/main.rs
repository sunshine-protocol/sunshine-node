use crate::command::*;
use crate::error::Error;
use crate::runtime::{AccountId, Extra, OrgId, Runtime, Shares, Signature};
use clap::Clap;
use core::convert::TryInto;
use exitfailure::ExitDisplay;
use ipfs_embed::{Config, Store};
use ipld_block_builder::{BlockBuilder, Codec};
use keybase_keystore::bip39::{Language, Mnemonic};
use keybase_keystore::{DeviceKey, KeyStore, Password};
use sp_keyring::AccountKeyring;
use std::path::PathBuf;
use substrate_subxt::sp_core::sr25519;
use substrate_subxt::ClientBuilder;

mod command;
mod error;
mod runtime;

#[async_std::main]
async fn main() -> Result<(), ExitDisplay<Error>> {
    Ok(run().await?)
}

struct Paths {
    _root: PathBuf,
    keystore: PathBuf,
    db: PathBuf,
}

impl Paths {
    fn new(root: Option<PathBuf>) -> Result<Self, Error> {
        let root = if let Some(root) = root {
            root
        } else {
            dirs::config_dir()
                .ok_or(Error::ConfigDirNotFound)?
                .join("sunshine-cli")
        };
        let keystore = root.join("keystore");
        let db = root.join("db");
        Ok(Paths {
            _root: root,
            keystore,
            db,
        })
    }
}

type Client = sunshine_client::SunClient<Runtime, Signature, Extra, sr25519::Pair, Store>;

async fn run() -> Result<(), Error> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    // initialize requisite storage utilities
    let paths = Paths::new(opts.path)?;
    let keystore = KeyStore::new(&paths.keystore);
    // initialize keystore with alice's keys
    let alice_seed: [u8; 32] = AccountKeyring::Alice.into();
    keystore.initialize(
        &DeviceKey::from_seed(alice_seed),
        &Password::from("password".to_string()),
    )?;
    let subxt = ClientBuilder::<Runtime>::new().build().await?;
    let config = Config::from_path(&paths.db).map_err(ipfs_embed::Error::Sled)?;
    let store = Store::new(config)?;
    let codec = Codec::new();
    let ipld = BlockBuilder::new(store, codec);
    // initialize new client from storage utilities
    let client = Client::new(keystore, subxt.clone(), ipld);
    // match on the passed in command
    match opts.cmd {
        SubCommand::Key(KeyCommand { cmd }) => match cmd {
            KeySubCommand::Set(KeySetCommand {
                force,
                suri,
                paperkey,
            }) => {
                if client.has_device_key() && !force {
                    return Err(Error::HasDeviceKey);
                }
                let password = ask_for_password("Please enter a new password (8+ characters):\n")?;
                if password.expose_secret().len() < 8 {
                    return Err(Error::PasswordTooShort);
                }
                let dk = if paperkey {
                    let mnemonic = ask_for_phrase("Please enter your backup phrase:").await?;
                    DeviceKey::from_mnemonic(&mnemonic).map_err(|_| Error::InvalidMnemonic)?
                } else {
                    if let Some(suri) = &suri {
                        DeviceKey::from_seed(suri.0)
                    } else {
                        DeviceKey::generate()
                    }
                };
                let account_id = client.set_device_key(&dk, &password, force)?;
                let account_id_str = account_id.to_string();
                println!("Your device id is {}", &account_id_str);
            }
            _ => {
                println!("lock and unlock left unimplemented for now");
            }
        },
        SubCommand::Wallet(WalletCommand { cmd }) => match cmd {
            WalletSubCommand::IssueShares(WalletIssueSharesCommand {
                organization,
                who,
                shares,
            }) => {
                let org: OrgId = organization.try_into()?;
                let recipient: AccountId =
                    who.into_account().ok_or(Error::AccountIdConversionFailed)?;
                let new_shares_minted: Shares = shares.try_into()?;
                let event = client
                    .issue_shares(org, recipient, new_shares_minted)
                    .await?;
                println!(
                    "{} shares issued for {} account in {} org",
                    event.shares, event.who, event.organization
                );
            }
            WalletSubCommand::ReserveShares(WalletReserveSharesCommand { organization, who }) => {
                let org: OrgId = organization.try_into()?;
                let owner_of_reserved_shares: AccountId =
                    who.into_account().ok_or(Error::AccountIdConversionFailed)?;
                let event = client
                    .reserve_shares(org, &owner_of_reserved_shares)
                    .await?;
                println!(
                    "{} shares reserved for {} account in {} org",
                    event.amount_reserved, event.who, event.organization
                );
            }
            WalletSubCommand::LockShares(WalletLockSharesCommand { organization, who }) => {
                let org: OrgId = organization.try_into()?;
                let owner_of_locked_shares: AccountId =
                    who.into_account().ok_or(Error::AccountIdConversionFailed)?;
                let event = client.lock_shares(org, &owner_of_locked_shares).await?;
                println!(
                    "shares locked for {} account in {} org",
                    event.who, event.organization
                );
            }
        },
        SubCommand::Run => loop {
            async_std::task::sleep(std::time::Duration::from_millis(100)).await
        },
    }
    Ok(())
}

fn ask_for_password(prompt: &str) -> Result<Password, Error> {
    Ok(Password::from(rpassword::prompt_password_stdout(prompt)?))
}

async fn ask_for_phrase(prompt: &str) -> Result<Mnemonic, Error> {
    println!("{}", prompt);
    let mut words = Vec::with_capacity(24);
    while words.len() < 24 {
        let mut line = String::new();
        async_std::io::stdin().read_line(&mut line).await?;
        for word in line.split(' ') {
            words.push(word.trim().to_string());
        }
    }
    println!("");
    Ok(Mnemonic::from_phrase(&words.join(" "), Language::English)
        .map_err(|_| Error::InvalidMnemonic)?)
}