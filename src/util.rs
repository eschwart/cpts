use crate::consts::*;
use jupiter_swap_api_client::{
    quote::QuoteResponse,
    swap::{
        SwapInstructionsResponse, SwapInstructionsResponseInternal, SwapRequest, UiSimulationError,
    },
    transaction_config::TransactionConfig,
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub fn lamports_to_sol(lamports: u64) -> String {
    solana_cli_output::display::build_balance_message(lamports, false, true)
}

pub fn quote(
    rpc: &reqwest::blocking::Client,
    wallet_addr: Pubkey,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    amount: u64,
) -> anyhow::Result<SwapInstructionsResponse> {
    let res = rpc.get(format!("{JUP_API}/quote?inputMint={input_mint}&outputMint={output_mint}&amount={amount}&slippageBps=200&restrictIntermediateTokens=true")).send()?;

    // exit early, if error
    if !res.status().is_success() {
        let ui_sim_err = res.json::<UiSimulationError>()?;
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

    let quote_response = res.json::<QuoteResponse>()?;

    let swap_request = SwapRequest {
        user_public_key: wallet_addr,
        quote_response,
        config: TransactionConfig {
            wrap_and_unwrap_sol: true,
            ..Default::default()
        },
    };

    let res = rpc
        .post(format!("{JUP_API}/swap-instructions"))
        .json(&swap_request)
        .send()?;
    let swap_ixs_internal = res.json::<SwapInstructionsResponseInternal>()?;
    Ok(swap_ixs_internal.into())
}

pub fn get_ixs(swap_ixs: SwapInstructionsResponse) -> Vec<Instruction> {
    let mut ixs: Vec<Instruction> = Vec::new();

    if let Some(ix) = swap_ixs.token_ledger_instruction {
        ixs.push(ix);
    }

    ixs.extend(swap_ixs.compute_budget_instructions);
    ixs.extend(swap_ixs.setup_instructions);

    ixs.push(swap_ixs.swap_instruction);

    // if let Some(ix) = swap_ixs.cleanup_instruction {
    //     ixs.push(ix);
    // }

    ixs.extend(swap_ixs.other_instructions);

    ixs
}
