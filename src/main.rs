mod cfg;
mod consts;
mod util;

use cfg::*;
use util::*;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::time::Instant;

async fn run() -> anyhow::Result<()> {
    let cfg = Config::new()?;
    let t = Instant::now();

    println!(
        "cpts ({})",
        if cfg.is_simulation() { "mock" } else { "real" }
    );

    // TODO - `processed` or `confirmed` (most likely) or `finalized`
    let client_rpc = RpcClient::new_with_commitment(cfg.rpc_url(), CommitmentConfig::confirmed());
    let client_req = reqwest::Client::new();

    let starting_balance = client_rpc.get_balance(&cfg.wallet_addr()).await?;
    println!(
        "{} : {}\n",
        cfg.wallet_addr(),
        lamports_to_sol(starting_balance),
    );

    // exit early if balance flag was provided
    if cfg.balance() {
        let ata = spl_associated_token_account::get_associated_token_address(
            &cfg.wallet_addr(),
            &cfg.mint(),
        );
        let balance_resp = client_rpc.get_token_account_balance(&ata).await?;
        println!("ATA Balance: {} units", balance_resp.amount.parse::<u64>()?);
        return Ok(());
    }

    // buying or selling
    let (from, to) = if cfg.is_buy() {
        (spl_token::native_mint::ID, cfg.mint())
    } else {
        (cfg.mint(), spl_token::native_mint::ID)
    };

    // number of units
    let amount = if let Some(units) = cfg.units() {
        units
    } else {
        let ata = spl_associated_token_account::get_associated_token_address(
            &cfg.wallet_addr(),
            &cfg.mint(),
        );
        let balance_resp = client_rpc.get_token_account_balance(&ata).await?;
        balance_resp.amount.parse()?
    };

    if amount == 0 {
        return Err(anyhow::Error::msg(format!(
            "You don't own any {}.",
            cfg.mint()
        )));
    }

    let request = order(
        &client_req,
        cfg.wallet_addr(),
        cfg.private_key(),
        &from,
        &to,
        amount,
        cfg.jupiter_api_key(),
    )
    .await?;

    // attempt several times
    for i in 0..3 {
        if cfg.is_simulation() {
            todo!()
        } else {
            match execute(&client_req, &request, cfg.jupiter_api_key()).await {
                Ok(s) => {
                    println!("[{i}] [{:?}] {}", t.elapsed(), s);
                    break;
                }
                Err(e) => {
                    eprintln!("[{i}] {}", e);
                    continue;
                }
            }
        }
    }
    // let new_balance = client_rpc.get_balance(&cfg.wallet_addr()).await?;
    // let change = new_balance as i64 - starting_balance as i64;
    // println!(
    //     "\nBALANCE: {} -> {} ({}{} change)\n",
    //     lamports_to_sol(starting_balance),
    //     lamports_to_sol(new_balance),
    //     if change.is_positive() { '+' } else { '-' },
    //     lamports_to_sol(change.unsigned_abs())
    // );
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("ERROR : {}", e)
    }
}
