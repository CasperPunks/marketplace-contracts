use alloc::{string::String, vec};

use crate::constants::*;
use alloc::boxed::Box;

use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter};

fn transfer_owner() -> EntryPoint {
    EntryPoint::new(
        String::from(TRANSFER_OWNER_ENTRY_POINT_NAME),
        vec![Parameter::new(ARG_CONTRACT_OWNER, CLType::Key)],
        CLType::Key,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
fn change_fee() -> EntryPoint {
    EntryPoint::new(
        String::from(CHANGE_FEE_ENTRY_POINT_NAME),
        vec![Parameter::new(ARG_MARKET_FEE, CLType::U256)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn sell() -> EntryPoint {
    EntryPoint::new(
        String::from(SELL_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_NFT_CONTRACT_HASH, CLType::Key),
            Parameter::new(ARG_MINIMUM_OFFER, CLType::U256),
            Parameter::new(ARG_TOKEN_ID, CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
fn buy() -> EntryPoint {
    EntryPoint::new(
        String::from(BUY_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_NFT_CONTRACT_HASH, CLType::Key),
            Parameter::new("amount", CLType::U256),
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_BUYER, CLType::Key),
            Parameter::new("src_purse", CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn accept_price() -> EntryPoint {
    EntryPoint::new(
        String::from(ACCEPT_PRICE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_ACCEPTED_BIDDER, CLType::Key),
            Parameter::new(ARG_ACCEPTED_PRICE, CLType::U256),
            Parameter::new(ARG_TOKEN_ID, CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn bid() -> EntryPoint {
    EntryPoint::new(
        String::from(BID_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_NFT_CONTRACT_HASH, CLType::Key),
            Parameter::new("amount", CLType::U256),
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_BUYER, CLType::Key),
            Parameter::new("src_purse", CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn revoke_bid() -> EntryPoint {
    EntryPoint::new(
        String::from(REVOKE_BID_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_NFT_CONTRACT_HASH, CLType::Key),
            Parameter::new(ARG_TOKEN_ID, CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn increase_bid() -> EntryPoint {
    EntryPoint::new(
        String::from(INCREASE_BID_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_NFT_CONTRACT_HASH, CLType::Key),
            Parameter::new("amount", CLType::U256),
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_BIDDER, CLType::Key),
            Parameter::new("src_purse", CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn revoke_sell() -> EntryPoint {
    EntryPoint::new(
        String::from(REVOKE_OFFER_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_NFT_CONTRACT_HASH, CLType::Key),
            Parameter::new(ARG_TOKEN_ID, CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn change_price() -> EntryPoint {
    EntryPoint::new(
        String::from(CHANGE_PRICE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_TOKEN_ID, CLType::String),
            Parameter::new(ARG_MINIMUM_OFFER, CLType::U256),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn emergency_withdraw_cspr() -> EntryPoint {
    EntryPoint::new(
        String::from(EMEGENCY_WITHDRAW_CSPR),
        vec![Parameter::new(AMOUNT_RUNTIME_ARG_NAME, CLType::U256)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn emergency_withdraw_nfts() -> EntryPoint {
    EntryPoint::new(
        String::from(EMEGENCY_WITHDRAW_NFTS),
        vec![
            Parameter::new(ARG_TOKEN_IDS, CLType::List(Box::new(CLType::String))),
            Parameter::new(ARG_NFT_CONTRACT_HASH, CLType::Key),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn set_support_token() -> EntryPoint {
    EntryPoint::new(
        String::from(SET_SUPPORTED_TOKEN_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_NFT_CONTRACT_HASH, CLType::Key),
            Parameter::new(ARG_NFT_ENABLED, CLType::Bool),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn init() -> EntryPoint {
    EntryPoint::new(
        String::from(INIT_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_CONTRACT_HASH, CLType::Key),
            Parameter::new(ARG_CONTRACT_OWNER, CLType::Key),
            Parameter::new(ARG_MARKET_FEE_RECEIVER, CLType::Key),
            Parameter::new(ARG_MARKET_FEE, CLType::U256),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of ERC20 token entry points.
pub(crate) fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(transfer_owner());
    entry_points.add_entry_point(init());
    entry_points.add_entry_point(change_fee());
    entry_points.add_entry_point(revoke_sell());
    entry_points.add_entry_point(sell());
    entry_points.add_entry_point(change_price());
    entry_points.add_entry_point(buy());
    entry_points.add_entry_point(set_support_token());
    entry_points.add_entry_point(bid());
    entry_points.add_entry_point(revoke_bid());
    entry_points.add_entry_point(increase_bid());
    entry_points.add_entry_point(emergency_withdraw_nfts());
    entry_points.add_entry_point(emergency_withdraw_cspr());
    entry_points.add_entry_point(accept_price());
    entry_points
}
