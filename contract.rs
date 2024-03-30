use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use serde::{Deserialize, Serialize};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

const BURN_ADDRESS: &str = "terra1sk06e3dyexuq4shw77y3dsv480xv42mq73anxu";
const CW20_CONTRACT_ADDRESS: &str = "terra1fvd5fvye7kgk0gudks6qtjz5nv6hcrdyukke59uj493w6496rv6sk87wpu";
const LP_PROVIDER_ADDRESS: &str = "terra1hchcv5glp9aqgwp88lpw45htssz3g4q3m0rear";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExecuteMsg {}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {

    let uluna_sent = info
        .funds
        .iter()
        .find(|coin| coin.denom == "uluna")
        .map_or(Uint128::zero(), |coin| coin.amount);

    if uluna_sent.is_zero() {
        return Err(StdError::generic_err("No uluna sent"));
    }

    let gas_fee = uluna_sent * Uint128::from(1u128) / Uint128::from(100u128);
    let amount_to_forward = uluna_sent - gas_fee;

    if amount_to_forward <= Uint128::zero() {
        return Err(StdError::generic_err("Insufficient uluna sent to cover gas fee"));
    }
    
    let fifty_percent_amount_to_forward = amount_to_forward / Uint128::from(2u128);
    
    let send_msg_to_lp_provider = BankMsg::Send {
        to_address: LP_PROVIDER_ADDRESS.to_string(),
        amount: vec![Coin {
            denom: "uluna".to_string(),
            amount: fifty_percent_amount_to_forward,
        }],
    };
    
    let send_msg_to_burn_address = BankMsg::Send {
        to_address: BURN_ADDRESS.to_string(),
        amount: vec![Coin {
            denom: "uluna".to_string(),
            amount: fifty_percent_amount_to_forward,
        }],
    };

    let cw20_transfer_msg = WasmMsg::Execute {
        contract_addr: CW20_CONTRACT_ADDRESS.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount: amount_to_forward,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(send_msg_to_lp_provider))
        .add_message(CosmosMsg::Bank(send_msg_to_burn_address))
        .add_message(CosmosMsg::Wasm(cw20_transfer_msg))
        .add_attribute("method", "execute"))
}
