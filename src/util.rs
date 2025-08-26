use std::str::FromStr;

use crate::consts::*;
use base64::Engine;
use jupiter_swap_api_client::swap::UiSimulationError;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::VersionedTransaction,
};

pub fn lamports_to_sol(lamports: u64) -> String {
    solana_cli_output::display::build_balance_message(lamports, false, true)
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct OrderResponse {
    transaction: String,
    requestId: String,
    slippageBps: u64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteRequest {
    signedTransaction: String,
    requestId: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteSuccess {
    signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteFail {
    error: String,
    code: u64,
}

pub async fn execute(
    client_req: &reqwest::Client,
    request: &ExecuteRequest,
    jupiter_api_key: &str,
) -> anyhow::Result<Signature> {
    let res = client_req
        .post(format!("{JUP_API}/execute"))
        .json(request)
        .header("x-api-key", jupiter_api_key)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(Signature::from_str(
            &res.json::<ExecuteSuccess>().await?.signature,
        )?)
    } else {
        Err(anyhow::Error::msg(res.json::<ExecuteFail>().await?.error))
    }
}

pub async fn order(
    client_req: &reqwest::Client,
    wallet_addr: Pubkey,
    private_key: &Keypair,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    amount: u64,
    jupiter_api_key: &str,
) -> anyhow::Result<ExecuteRequest> {
    let res = client_req.get(format!("{JUP_API}/order?inputMint={input_mint}&outputMint={output_mint}&amount={amount}&taker={wallet_addr}&slippageBps=2000")).header("x-api-key", jupiter_api_key).send().await?;

    // exit early, if error
    if !res.status().is_success() {
        let ui_sim_err = res.json::<UiSimulationError>().await?;
        let ui_sim_err_val = serde_json::to_value(ui_sim_err)?;
        let error = ui_sim_err_val
            .get("error")
            .cloned()
            .ok_or(anyhow::Error::msg("Failed to parse error message."))?;

        return Err(anyhow::Error::msg(
            error
                .as_str()
                .map(ToString::to_string)
                .ok_or(anyhow::Error::msg(
                    "Failed to parse error message as string.",
                ))?,
        ));
    }

    let quote_response = res.json::<OrderResponse>().await.unwrap();

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(quote_response.transaction)
        .unwrap();
    let vtx_unsigned = bincode::deserialize::<VersionedTransaction>(bytes.as_slice()).unwrap();

    let vtx = VersionedTransaction::try_new(vtx_unsigned.message, &[private_key]).unwrap();

    let bytes = bincode::serialize(&vtx).unwrap();
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);

    Ok(ExecuteRequest {
        signedTransaction: b64,
        requestId: quote_response.requestId,
    })
}
