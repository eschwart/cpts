mod cfg;
mod consts;
mod util;

use cfg::*;
use consts::*;
use util::*;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, transaction::Transaction};
use std::time::Instant;

fn run() -> anyhow::Result<()> {
    let cfg = Config::new()?;
    let t = Instant::now();

    println!(
        "cpts ({})",
        if cfg.is_simulation() { "mock" } else { "real" }
    );

    // TODO - `processed` or `confirmed` (most likely) or `finalized`
    let client_rpc = RpcClient::new_with_commitment(cfg.rpc_url(), CommitmentConfig::confirmed());
    let client_req = reqwest::blocking::Client::new();

    let starting_balance = client_rpc.get_balance(&cfg.wallet_addr())?;
    println!(
        "BALANCE : {}\nWALLET  : {}\n",
        lamports_to_sol(starting_balance),
        cfg.wallet_addr(),
    );

    // buying or selling
    let (from, to) = if cfg.is_buy() {
        (WSOL, cfg.mint())
    } else {
        (cfg.mint(), WSOL)
    };

    // number of units
    let amount = if let Some(units) = cfg.units() {
        units
    } else {
        let ata = spl_associated_token_account::get_associated_token_address(
            &cfg.wallet_addr(),
            &cfg.mint(),
        );
        let balance_resp = client_rpc.get_token_account_balance(&ata)?;
        balance_resp.amount.parse()?
    };

    if amount == 0 {
        return Err(anyhow::Error::msg(format!(
            "You don't own any {}.",
            cfg.mint()
        )));
    }

    let quote_res = quote(&client_req, cfg.wallet_addr(), &from, &to, amount)?;
    let ixs = get_ixs(quote_res);

    // retrieve latest blockhash
    let blockhash = client_rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&cfg.wallet_addr()),
        &[cfg.private_key()],
        blockhash,
    );

    if cfg.is_simulation() {
        let res = client_rpc.simulate_transaction(&tx)?;
        match res.value.err {
            None => println!("SUCCESS [{:?} elapsed]: {:?}\n", t.elapsed(), res.value),
            Some(e) => println!(
                "FAILURE [{:?} elapsed]: {:?}\n{:#?}\n",
                t.elapsed(),
                e,
                res.value.logs
            ),
        }
    } else {
        let res = client_rpc.send_and_confirm_transaction(&tx);
        match res {
            Ok(sig) => println!("SUCCESS [{:?} elapsed]: {}", t.elapsed(), sig),
            Err(e) => println!("FAILURE [{:?} elapsed]: {:?}", t.elapsed(), e),
        }
    }
    let new_balance = client_rpc.get_balance(&cfg.wallet_addr())?;
    let change = new_balance as i64 - starting_balance as i64;
    println!(
        "\nBALANCE: {} -> {} ({}{} change)\n",
        lamports_to_sol(starting_balance),
        lamports_to_sol(new_balance),
        if change.is_positive() { '+' } else { '-' },
        lamports_to_sol(change.unsigned_abs())
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR   : {}", e)
    }
}
