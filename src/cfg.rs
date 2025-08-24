use std::{env::current_exe, fs::File, str::FromStr};

use clap::Parser;
use serde::Deserialize;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

fn parse(s: &str) -> Result<(bool, u64), anyhow::Error> {
    let v = s.parse::<i64>()?;
    if v != 0 {
        Ok((v.is_positive(), v.unsigned_abs()))
    } else {
        Err(anyhow::Error::msg("Units can't be zero."))
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Creds {
    wallet_address: String,
    wallet_private_key: String,
    quicknode_rpc_url: String,
}

/// Execute buys/sells (Quicknodes and Jupiter).
#[derive(Clone, Debug, Parser)]
struct ProgramConfig {
    /// Shit-coin mint (address).
    #[arg(short, long)]
    mint: Pubkey,

    /// Number of units (amount).
    #[arg(short, long, default_value = None, allow_hyphen_values = true, value_parser = parse)]
    units: Option<(bool, u64)>,

    /// Dry-run.
    #[arg(short, long, default_value_t)]
    simulate: bool,
}

#[derive(Debug)]
pub struct Config {
    mint: Pubkey,
    units: Option<(bool, u64)>,
    is_simulation: bool,
    wallet_address: Pubkey,
    wallet_private_key: Keypair,
    quicknode_rpc_url: String,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        // retrive the path of the relative config.json file.
        let path = current_exe()?
            .parent()
            .ok_or(anyhow::Error::msg(
                "Failed to retrieve current_exe path directory.",
            ))?
            .join("config.json");

        // open the config.json file
        let rdr = File::open(path).map_err(|e| {
            if let std::io::ErrorKind::NotFound = e.kind() {
                anyhow::Error::msg("Failed to open file 'config.json'.")
            } else {
                anyhow::Error::from(e)
            }
        })?;

        // deserialize config.json file into [`Creds`]
        let Creds {
            wallet_address,
            wallet_private_key,
            quicknode_rpc_url,
        } = serde_json::from_reader::<_, Creds>(rdr)?;

        let wallet_address = Pubkey::from_str(&wallet_address)?;
        let wallet_private_key = Keypair::from_base58_string(&wallet_private_key);

        let cfg = ProgramConfig::parse();

        Ok(Self {
            mint: cfg.mint,
            units: cfg.units,
            is_simulation: cfg.simulate,
            wallet_address,
            wallet_private_key,
            quicknode_rpc_url,
        })
    }

    pub const fn mint(&self) -> Pubkey {
        self.mint
    }

    pub fn is_buy(&self) -> bool {
        self.units.is_some_and(|(b, ..)| b)
    }

    pub fn units(&self) -> Option<u64> {
        self.units.map(|(.., v)| v)
    }

    pub const fn is_simulation(&self) -> bool {
        self.is_simulation
    }

    pub const fn wallet_addr(&self) -> Pubkey {
        self.wallet_address
    }

    pub const fn private_key(&self) -> &Keypair {
        &self.wallet_private_key
    }

    pub const fn rpc_url(&self) -> &str {
        self.quicknode_rpc_url.as_str()
    }
}
