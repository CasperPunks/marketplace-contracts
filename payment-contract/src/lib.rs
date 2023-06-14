#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;
mod converters;
mod error;

use crate::converters::u512_to_u256;
use crate::error::Error;
use contract::contract_api::{account, runtime, system};
use contract::unwrap_or_revert::UnwrapOrRevert;
use types::{runtime_args, ContractHash, HashAddr, Key, RuntimeArgs, URef, U256, U512};

#[no_mangle]
pub extern "C" fn call() {
    let deposit_amount: U512 = runtime::get_named_arg("amount");
    let deposit_entry_point_name: String = runtime::get_named_arg("deposit_entry_point_name");

    let new_purse = system::create_purse();
    system::transfer_from_purse_to_purse(
        account::get_main_purse(),
        new_purse,
        deposit_amount,
        None,
    )
    .unwrap_or_revert_with(Error::ExcessiveAmount);

    let deposit_entry_point_args = retrieve_deposit_entry_point_name_args(
        deposit_entry_point_name.clone(),
        new_purse,
        u512_to_u256(deposit_amount),
    );

    if deposit_entry_point_args.is_empty() {
        runtime::revert(Error::InvalidDepositEntryPointName);
    }

    let nft_contract_hash: Key = runtime::get_named_arg::<Key>("nft_contract_hash");

    let (arg, register) = get_register_owner_args(deposit_entry_point_name.clone());
    if register {
        let (_, _): (String, URef) = runtime::call_contract(
            ContractHash::new(nft_contract_hash.into_hash().unwrap_or_revert()),
            "register_owner",
            arg,
        );
    }

    let marketplace_input: Key = runtime::get_named_arg("marketplace_hash");

    let mk_contract_hash_addr: HashAddr = marketplace_input.into_hash().unwrap_or_revert();
    let mk_contract_hash: ContractHash = ContractHash::new(mk_contract_hash_addr);

    runtime::call_contract::<()>(
        mk_contract_hash,
        &deposit_entry_point_name,
        deposit_entry_point_args,
    )
}

fn retrieve_deposit_entry_point_name_args(
    deposit_entry_point_name: String,
    purse: URef,
    amount: U256,
) -> RuntimeArgs {
    if deposit_entry_point_name == "buy" {
        retrieve_buy_args(purse, amount)
    } else if deposit_entry_point_name == "bid" {
        retrieve_bid_args(purse, amount)
    } else if deposit_entry_point_name == "increase_bid" {
        retrieve_increase_bid_args(purse, amount)
    } else {
        runtime_args! {}
    }
}

fn get_register_owner_args(deposit_entry_point_name: String) -> (RuntimeArgs, bool) {
    if deposit_entry_point_name == "buy" {
        (
            runtime_args! {
                "token_owner" => runtime::get_named_arg::<Key>("buyer")
            },
            true,
        )
    } else if deposit_entry_point_name == "bid" || deposit_entry_point_name == "increase_bid" {
        (
            runtime_args! {
                "token_owner" => runtime::get_named_arg::<Key>("bidder")
            },
            true,
        )
    } else {
        (runtime_args! {}, false)
    }
}

fn retrieve_buy_args(src_purse: URef, amount: U256) -> RuntimeArgs {
    runtime_args! {
        "nft_contract_hash" => runtime::get_named_arg::<Key>("nft_contract_hash"),
        "token_id" => runtime::get_named_arg::<String>("token_id"),
        "amount" => amount,
        "src_purse" => src_purse,
        "buyer" => runtime::get_named_arg::<Key>("buyer"),
    }
}

fn retrieve_bid_args(src_purse: URef, amount: U256) -> RuntimeArgs {
    runtime_args! {
        "amount" => amount,
        "src_purse" => src_purse,
        "nft_contract_hash" => runtime::get_named_arg::<Key>("nft_contract_hash"),
        "token_id" => runtime::get_named_arg::<String>("token_id"),
        "bidder" => runtime::get_named_arg::<Key>("bidder"),
    }
}
fn retrieve_increase_bid_args(src_purse: URef, amount: U256) -> RuntimeArgs {
    runtime_args! {
        "amount" => amount,
        "src_purse" => src_purse,
        "nft_contract_hash" => runtime::get_named_arg::<Key>("nft_contract_hash"),
        "token_id" => runtime::get_named_arg::<String>("token_id"),
        "bidder" => runtime::get_named_arg::<Key>("bidder"),
    }
}
