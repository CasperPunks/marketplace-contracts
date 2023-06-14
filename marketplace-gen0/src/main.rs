#![no_main]
#![no_std]
#![feature(type_ascription)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

mod address;
pub mod constants;
mod entry_points;
mod error;
mod events;
mod helpers;
pub mod named_keys;
use serde::{Deserialize, Serialize};

use crate::constants::*;
use crate::error::Error;
use crate::helpers::*;
use alloc::{
    string::{String, ToString},
    vec,
    vec::*,
};
use casper_contract::{
    contract_api::{
        runtime,
        // runtime::print,
        storage,
        system::{transfer_from_purse_to_account, transfer_from_purse_to_purse},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr, bytesrepr::FromBytes, bytesrepr::ToBytes, contracts::NamedKeys, runtime_args,
    CLType, CLTyped, ContractHash, ContractPackageHash, HashAddr, Key, RuntimeArgs, URef, U256,
};
use events::MarketPlaceEvent;
use helpers::{get_immediate_caller_key, get_self_key};
const _FEE_DIVISOR: u64 = 10000;

// use k256::ecdsa::VerifyingKey;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct SellingInMarket {
    token_id: String,
    nft_contract: Key,
    offeror: Option<Key>, //token seller
    minimum_offer: U256,  // min price in WCSPR
    is_active: bool,
    bidder: Vec<Key>,
    bidding_price: Vec<U256>,
}

impl ToBytes for SellingInMarket {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.token_id.to_bytes()?);
        result.extend(self.nft_contract.to_bytes()?);
        result.extend(self.offeror.to_bytes()?);
        result.extend(self.minimum_offer.to_bytes()?);
        result.extend(self.is_active.to_bytes()?);
        result.extend(self.bidder.to_bytes()?);
        result.extend(self.bidding_price.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.token_id.serialized_length()
            + self.nft_contract.serialized_length()
            + self.offeror.serialized_length()
            + self.minimum_offer.serialized_length()
            + self.is_active.serialized_length()
            + self.bidder.serialized_length()
            + self.bidding_price.serialized_length()
    }
}

impl FromBytes for SellingInMarket {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (token_id, remainder) = String::from_bytes(bytes)?;
        let (nft_contract, remainder) = Key::from_bytes(remainder)?;
        let (offeror, remainder) = Option::<Key>::from_bytes(remainder)?;
        let (minimum_offer, remainder) = U256::from_bytes(remainder)?;
        let (is_active, remainder) = bool::from_bytes(remainder)?;
        let (bidder, remainder) = Vec::<Key>::from_bytes(remainder)?;
        let (bidding_price, remainder) = Vec::<U256>::from_bytes(remainder)?;

        let ret = SellingInMarket {
            token_id,
            nft_contract,
            offeror,       //token seller
            minimum_offer, // min price in WCSPR
            is_active,
            bidder,
            bidding_price,
        };
        Ok((ret, remainder))
    }
}

impl CLTyped for SellingInMarket {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

#[no_mangle]
pub extern "C" fn init() {
    if get_key::<Key>(CONTRACT_HASH_KEY_NAME).is_some() {
        runtime::revert(Error::ContractAlreadyInitialized);
    }
    let contract_hash: Key = runtime::get_named_arg(ARG_CONTRACT_HASH);

    let contract_owner: Key = runtime::get_named_arg(ARG_CONTRACT_OWNER);

    let contract_fee_receiver: Key = runtime::get_named_arg(ARG_MARKET_FEE_RECEIVER);

    let market_fee: U256 = runtime::get_named_arg(ARG_MARKET_FEE);
    let nft_contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);

    runtime::put_key(
        CONTRACT_HASH_KEY_NAME,
        storage::new_uref(contract_hash).into(),
    );

    runtime::put_key(
        CONTRACT_OWNER_KEY_NAME,
        storage::new_uref(contract_owner).into(),
    );

    runtime::put_key(
        MARKET_FEE_RECEIVER,
        storage::new_uref(contract_fee_receiver).into(),
    );

    runtime::put_key(MARKET_FEE, storage::new_uref(market_fee as U256).into());
    runtime::put_key(
        TOKEN_CONTRACT_SUPPORT,
        storage::new_uref(nft_contract_hash).into(),
    );

    storage::new_dictionary(SELLING_IN_MARKET)
        .unwrap_or_revert_with(Error::FailedToCreateDictionary);
}

#[no_mangle]
fn call() {
    let contract_name: String = runtime::get_named_arg(MARKET_CONTRACT_NAME);
    let contract_hash_key_name = contract_name.clone();
    let _contract_package_hash_key_name = contract_name + "_package_name";
    let contract_owner: Key = runtime::get_named_arg(ARG_CONTRACT_OWNER);
    let market_fee_receiver: Key = runtime::get_named_arg(ARG_MARKET_FEE_RECEIVER);
    let market_fee: U256 = runtime::get_named_arg(ARG_MARKET_FEE);
    let nft_contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);

    let (contract_package_hash, _access_uref) = storage::create_contract_package_at_hash();

    let named_keys: NamedKeys = named_keys::default(
        contract_owner,
        market_fee_receiver,
        market_fee,
        contract_package_hash,
        None,
    );

    let (contract_hash, _version) = storage::add_contract_version(
        contract_package_hash: ContractPackageHash,
        entry_points::default(),
        named_keys: NamedKeys,
    );
    runtime::put_key(contract_hash_key_name.as_str(), Key::from(contract_hash));
    runtime::put_key("mk_gen0_pk", Key::from(contract_package_hash));
    runtime::put_key("mk_gen0_pk_access", Key::from(_access_uref));

    // set_key(PUNK_MARKETPLACE_KEY_NAME, Key::from(contract_hash));

    runtime::call_contract::<()>(
        contract_hash,
        INIT_ENTRY_POINT_NAME,
        runtime_args! {
            ARG_CONTRACT_HASH => Key::from(contract_hash),
            ARG_CONTRACT_OWNER => contract_owner,
            ARG_MARKET_FEE_RECEIVER => market_fee_receiver,
            ARG_MARKET_FEE => market_fee,
            ARG_NFT_CONTRACT_HASH => nft_contract_hash
        },
    );
}

// fn get_selling_token(sell_index: u64) -> SellingInMarket {
//     let token_market_str =
//         get_dictionary_value_from_key::<String>(SELLING_IN_MARKET, &sell_index.to_string())
//             .unwrap_or_revert();
//     let token_market =
//         casper_serde_json_wasm::from_str::<SellingInMarket>(&token_market_str).unwrap();
//     token_market
// }

#[no_mangle]
pub extern "C" fn sell() {
    let contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    // Check if nft is supported or not
    check_enabled_nft(contract_hash: Key);
    // Take token_id from runtime
    let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);
    let seller: Key = helpers::get_immediate_caller_key();

    let minimum_offer: U256 = runtime::get_named_arg(ARG_MINIMUM_OFFER);

    if minimum_offer == U256::zero() {
        runtime::revert(Error::AskForMore);
    }

    set_selling(&contract_hash, &token_id, seller, minimum_offer);
}

#[no_mangle]
pub extern "C" fn change_price() {
    let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);
    let new_price: U256 = runtime::get_named_arg(ARG_MINIMUM_OFFER);

    let token_market =
        get_dictionary_value_from_key::<SellingInMarket>(SELLING_IN_MARKET, &token_id);

    let caller: Key = get_immediate_caller_key();
    let mut unwrap = token_market.unwrap();
    let seller_key: Key = unwrap.offeror.unwrap();

    if !unwrap.is_active {
        runtime::revert(Error::OfferInactive)
    }

    if seller_key != caller {
        runtime::revert(Error::InvalidAccount)
    }
    if !unwrap.bidding_price.is_empty() && new_price <= *unwrap.bidding_price.last().unwrap() {
        let contract_purse = helpers::get_uref(CONTRACT_PURSE);

        let bidder = *unwrap.bidder.last().unwrap();
        let bidding_price = *unwrap.bidding_price.last().unwrap();

        if bidding_price - new_price > U256::zero() {
            transfer_from_purse_to_account(
                contract_purse,
                bidder.into_account().unwrap(),
                u256_to_u512(bidding_price - new_price),
                None,
            )
            .unwrap_or_revert_with(Error::CanNotTransferCSPR);
        }

        do_trade_change_price(
            &mut unwrap,
            contract_purse,
            bidder,
            new_price,
            get_self_key(),
        )
    } else {
        unwrap.minimum_offer = new_price;
        write_dictionary_value_from_key(SELLING_IN_MARKET, &token_id, unwrap.clone());
        events::emit(&MarketPlaceEvent::Sell {
            nft_contract: unwrap.nft_contract,
            token_id: unwrap.token_id.to_string(),
            offeror: unwrap.offeror.unwrap(),
            minimum_offer: new_price,
            is_active: true,
        });
    }
}

#[no_mangle]
pub extern "C" fn accept_price() {
    let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);
    let accepted_price: U256 = runtime::get_named_arg(ARG_ACCEPTED_PRICE);
    let accepted_bidder: Key = runtime::get_named_arg(ARG_ACCEPTED_BIDDER);
    let token_market =
        get_dictionary_value_from_key::<SellingInMarket>(SELLING_IN_MARKET, &token_id);

    let caller: Key = get_immediate_caller_key();
    let mut unwrap = token_market.unwrap();
    let seller_key: Key = unwrap.offeror.unwrap();

    if seller_key != caller {
        runtime::revert(Error::InvalidAccount)
    }
    let contract_purse = helpers::get_uref(CONTRACT_PURSE);
    let old_index = unwrap
        .bidder
        .iter()
        .position(|x| x.into_account().unwrap() == accepted_bidder.into_account().unwrap());

    if unwrap.bidding_price[old_index.unwrap()] != accepted_price {
        runtime::revert(Error::InvalidContext);
    }

    // remove this bidder from bidder list

    unwrap.bidder.remove(old_index.unwrap());
    unwrap.bidding_price.remove(old_index.unwrap());
    do_trade(&mut unwrap, contract_purse, accepted_bidder, accepted_price)
}

#[no_mangle]
pub extern "C" fn buy() {
    // let nft_contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let nft_contract_hash = helpers::get_named_arg_with_user_errors::<Key>(
        ARG_NFT_CONTRACT_HASH,
        Error::MissingNFTContract,
        Error::InvalidNFTContract,
    )
    .unwrap_or_revert_with(Error::CanNotGetNFTContract); //Contract hash of NFT CASPERPUNK

    // Check if nft is supported or not
    check_enabled_nft(nft_contract_hash: Key);
    // let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);
    let token_id = helpers::get_named_arg_with_user_errors::<String>(
        ARG_TOKEN_ID,
        Error::MissingTokenID,
        Error::InvalidTokenIdentifier,
    )
    .unwrap_or_revert_with(Error::CanNotGetTokenId); //Contract hash of NFT CASPERPUNK

    let buyer = helpers::get_named_arg_with_user_errors::<Key>(
        ARG_BUYER,
        Error::MissingBuyer,
        Error::InvalidBuyer,
    )
    .unwrap_or_revert_with(Error::CanNotGetBuyer); //Contract hash of NFT CASPERPUNK

    let mut unwrap =
        get_dictionary_value_from_key::<SellingInMarket>(SELLING_IN_MARKET, &token_id).unwrap();

    if unwrap.nft_contract != nft_contract_hash || unwrap.token_id != token_id {
        runtime::revert(Error::InvalidInputTokenInfo);
    }

    if !unwrap.is_active {
        runtime::revert(Error::OfferInactive)
    }

    let needed_amount: U256 = unwrap.minimum_offer;

    let allowed_cspr_amount = helpers::get_named_arg_with_user_errors::<U256>(
        AMOUNT_RUNTIME_ARG_NAME,
        Error::MissingAmount,
        Error::InvalidAmount,
    )
    .unwrap_or_revert_with(Error::CannotGetAmount);
    if allowed_cspr_amount < needed_amount {
        runtime::revert(Error::NotEnoughAmount)
    }
    let src_purse: URef = helpers::get_named_arg_with_user_errors::<URef>(
        ARG_SRC_PURSE,
        Error::MissingSrcPurse,
        Error::InvalidSrcPurse,
    )
    .unwrap_or_revert_with(Error::CanNotGetUserPurse); //Contract hash of NFT CASPERPUNK
    let contract_purse = helpers::get_uref(CONTRACT_PURSE);

    transfer_from_purse_to_purse(src_purse, contract_purse, u256_to_u512(needed_amount), None)
        .unwrap_or_revert_with(Error::CanNotTransferCSPR);

    // check whether the buyer has a bid for this nft, if yes, refund it
    let buyer_index = unwrap
        .bidder
        .iter()
        .position(|x| x.into_account().unwrap() == buyer.into_account().unwrap());
    if let Some(..) = buyer_index {
        let buyer_index = buyer_index.unwrap();
        transfer_from_purse_to_account(
            contract_purse,
            unwrap.bidder[buyer_index].into_account().unwrap(),
            u256_to_u512(unwrap.bidding_price[buyer_index]),
            None,
        )
        .unwrap_or_revert_with(Error::CanNotTransferCSPR);
        unwrap.bidder.remove(buyer_index);
        unwrap.bidding_price.remove(buyer_index);
    }

    do_trade(&mut unwrap, contract_purse, buyer, needed_amount);
}

#[no_mangle]
pub extern "C" fn bid() {
    // let nft_contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let nft_contract_hash = helpers::get_named_arg_with_user_errors::<Key>(
        ARG_NFT_CONTRACT_HASH,
        Error::MissingNFTContract,
        Error::InvalidNFTContract,
    )
    .unwrap_or_revert_with(Error::CanNotGetNFTContract); //Contract hash of NFT CASPERPUNK

    // Check if nft is supported or not
    check_enabled_nft(nft_contract_hash: Key);
    // let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);
    let token_id = helpers::get_named_arg_with_user_errors::<String>(
        ARG_TOKEN_ID,
        Error::MissingTokenID,
        Error::InvalidTokenIdentifier,
    )
    .unwrap_or_revert_with(Error::CanNotGetTokenId); //Contract hash of NFT CASPERPUNK

    let bidder = helpers::get_named_arg_with_user_errors::<Key>(
        ARG_BIDDER,
        Error::MissingBidder,
        Error::InvalidBidder,
    )
    .unwrap_or_revert_with(Error::CanNotGetBuyer);

    let bidding_price = helpers::get_named_arg_with_user_errors::<U256>(
        AMOUNT_RUNTIME_ARG_NAME,
        Error::MissingBidder,
        Error::InvalidBidder,
    )
    .unwrap_or_revert_with(Error::CanNotGetBuyer); //Contract hash of NFT CASPERPUNK

    // Bid must be higher than 100 cspr
    if bidding_price < U256::from(100000000000u64) {
        runtime::revert(Error::InvalidAmount)
    }
    let token_market =
        get_dictionary_value_from_key::<SellingInMarket>(SELLING_IN_MARKET, &token_id);

    let mut unwrap = if let Some(..) = token_market {
        token_market.unwrap()
    } else {
        SellingInMarket {
            offeror: None,
            token_id: token_id.clone(),
            nft_contract: nft_contract_hash,
            minimum_offer: 0.into(),
            is_active: false,
            bidder: Vec::new(),
            bidding_price: Vec::new(),
        }
    };

    if unwrap.nft_contract != nft_contract_hash || unwrap.token_id != token_id {
        runtime::revert(Error::InvalidInputTokenInfo);
    }

    // check if bidder is already make bid
    if unwrap.bidder.contains(&bidder) {
        runtime::revert(Error::InvalidContext);
    }

    // Check if bidding price is higher than minimum_offer

    let src_purse: URef = helpers::get_named_arg_with_user_errors::<URef>(
        ARG_SRC_PURSE,
        Error::MissingSrcPurse,
        Error::InvalidSrcPurse,
    )
    .unwrap_or_revert_with(Error::CanNotGetUserPurse); //Contract hash of NFT CASPERPUNK
    let needed_amount: U256 = unwrap.minimum_offer;

    if unwrap.is_active && bidding_price >= needed_amount {
        let contract_purse = helpers::get_uref(CONTRACT_PURSE);

        transfer_from_purse_to_purse(src_purse, contract_purse, u256_to_u512(needed_amount), None)
            .unwrap_or_revert_with(Error::CanNotTransferCSPR);
        do_trade(&mut unwrap, contract_purse, bidder, needed_amount);
    } else {
        let contract_purse = helpers::get_uref(CONTRACT_PURSE);

        transfer_from_purse_to_purse(src_purse, contract_purse, u256_to_u512(bidding_price), None)
            .unwrap_or_revert_with(Error::CanNotTransferCSPR);

        // save to selling_in_market
        insert_new_bidder(&mut unwrap, bidder, bidding_price);

        write_dictionary_value_from_key(SELLING_IN_MARKET, &token_id, unwrap.clone());

        events::emit(&MarketPlaceEvent::Bid {
            nft_contract: nft_contract_hash,
            token_id,
            offeror: unwrap.offeror.unwrap_or_else(null_key),
            bidder,
            value: bidding_price,
        });
    }
}

#[no_mangle]
pub extern "C" fn revoke_bid() {
    // let nft_contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let nft_contract_hash = helpers::get_named_arg_with_user_errors::<Key>(
        ARG_NFT_CONTRACT_HASH,
        Error::MissingNFTContract,
        Error::InvalidNFTContract,
    )
    .unwrap_or_revert_with(Error::CanNotGetNFTContract); //Contract hash of NFT CASPERPUNK

    // Check if nft is supported or not
    check_enabled_nft(nft_contract_hash: Key);
    // let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);
    let token_id = helpers::get_named_arg_with_user_errors::<String>(
        ARG_TOKEN_ID,
        Error::MissingTokenID,
        Error::InvalidTokenIdentifier,
    )
    .unwrap_or_revert_with(Error::CanNotGetTokenId); //Contract hash of NFT CASPERPUNK

    let mut unwrap =
        get_dictionary_value_from_key::<SellingInMarket>(SELLING_IN_MARKET, &token_id).unwrap();

    if unwrap.nft_contract != nft_contract_hash || unwrap.token_id != token_id {
        runtime::revert(Error::InvalidInputTokenInfo);
    }

    let caller: Key = get_immediate_caller_key();

    let removed_index = unwrap
        .bidder
        .iter()
        .position(|x| x.into_account().unwrap() == caller.into_account().unwrap());

    let s_index = match removed_index {
        Some(index) => index,
        None => runtime::revert(Error::InvalidBidder),
    };

    let contract_purse = helpers::get_uref(CONTRACT_PURSE);
    // Check if bidding price is higher than minimum_offer
    transfer_from_purse_to_account(
        contract_purse,
        unwrap.bidder[s_index].into_account().unwrap(),
        u256_to_u512(unwrap.bidding_price[s_index]),
        None,
    )
    .unwrap_or_revert_with(Error::CanNotTransferCSPR);

    let ret_value_event = unwrap.bidding_price[s_index];
    // remove bidder and bidding_price from array
    unwrap.bidder.remove(s_index);
    unwrap.bidding_price.remove(s_index);

    write_dictionary_value_from_key(SELLING_IN_MARKET, &token_id, unwrap.clone());

    events::emit(&MarketPlaceEvent::RevokeBid {
        nft_contract: nft_contract_hash,
        token_id,
        offeror: unwrap.offeror.unwrap_or_else(null_key),
        bidder: caller,
        value: ret_value_event,
        is_revoke: true,
    });
}

#[no_mangle]
pub extern "C" fn increase_bid() {
    let nft_contract_hash = helpers::get_named_arg_with_user_errors::<Key>(
        ARG_NFT_CONTRACT_HASH,
        Error::MissingNFTContract,
        Error::InvalidNFTContract,
    )
    .unwrap_or_revert_with(Error::CanNotGetNFTContract);

    // Check if nft is supported or not
    check_enabled_nft(nft_contract_hash: Key);
    // let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);
    let token_id = helpers::get_named_arg_with_user_errors::<String>(
        ARG_TOKEN_ID,
        Error::MissingTokenID,
        Error::InvalidTokenIdentifier,
    )
    .unwrap_or_revert_with(Error::CanNotGetTokenId);

    let bidder = helpers::get_named_arg_with_user_errors::<Key>(
        ARG_BIDDER,
        Error::MissingBidder,
        Error::InvalidBidder,
    )
    .unwrap_or_revert_with(Error::CanNotGetBuyer);

    let added_price = helpers::get_named_arg_with_user_errors::<U256>(
        AMOUNT_RUNTIME_ARG_NAME,
        Error::MissingBidder,
        Error::InvalidBidder,
    )
    .unwrap_or_revert_with(Error::CanNotGetBuyer);

    let token_market =
        get_dictionary_value_from_key::<SellingInMarket>(SELLING_IN_MARKET, &token_id);

    // print(&added_price.to_string());
    let mut unwrap = token_market.unwrap();

    if unwrap.nft_contract != nft_contract_hash || unwrap.token_id != token_id {
        runtime::revert(Error::InvalidInputTokenInfo);
    }

    let contract_purse = helpers::get_uref(CONTRACT_PURSE);

    let src_purse: URef = helpers::get_named_arg_with_user_errors::<URef>(
        ARG_SRC_PURSE,
        Error::MissingSrcPurse,
        Error::InvalidSrcPurse,
    )
    .unwrap_or_revert_with(Error::CanNotGetUserPurse);

    // check if bidder is already made bid
    if !unwrap.bidder.contains(&bidder) {
        runtime::revert(Error::InvalidContext);
    }

    // get old bid of this bidder

    let old_index = unwrap
        .bidder
        .iter()
        .position(|x| x.into_account().unwrap() == bidder.into_account().unwrap());

    let old_bidding_price = unwrap.bidding_price[old_index.unwrap()];

    // Check if increased bidding price is higher than minimum_offer

    let increased_bidding_price = old_bidding_price + added_price;

    let needed_amount: U256 = unwrap.minimum_offer;

    if increased_bidding_price >= needed_amount && unwrap.is_active {
        // take cspr from bidder to contract
        transfer_from_purse_to_purse(
            src_purse,
            contract_purse,
            u256_to_u512(needed_amount - old_bidding_price),
            None,
        )
        .unwrap_or_revert_with(Error::CanNotTransferCSPR);

        unwrap.bidder.remove(old_index.unwrap());
        unwrap.bidding_price.remove(old_index.unwrap());
        do_trade(&mut unwrap, contract_purse, bidder, needed_amount);
    } else {
        if unwrap.bidder.len() != unwrap.bidding_price.len() {
            runtime::revert(Error::InvalidContext);
        }

        transfer_from_purse_to_purse(src_purse, contract_purse, u256_to_u512(added_price), None)
            .unwrap_or_revert_with(Error::CanNotTransferCSPR);

        unwrap.bidder.remove(old_index.unwrap() as usize);
        unwrap.bidding_price.remove(old_index.unwrap() as usize);

        // save to selling_in_market
        insert_new_bidder(&mut unwrap, bidder: Key, increased_bidding_price);

        write_dictionary_value_from_key(SELLING_IN_MARKET, &token_id, unwrap.clone());

        events::emit(&MarketPlaceEvent::Bid {
            nft_contract: nft_contract_hash,
            token_id,
            offeror: unwrap.offeror.unwrap_or_else(null_key),
            bidder,
            value: increased_bidding_price,
        });
    }
}

#[no_mangle]
pub extern "C" fn revoke_sell() {
    let contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);

    check_enabled_nft(contract_hash: Key);

    let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);

    let mut token_market =
        get_dictionary_value_from_key::<SellingInMarket>(SELLING_IN_MARKET, &token_id).unwrap();

    let caller = get_immediate_caller_key();
    if token_market.offeror.unwrap() != caller {
        runtime::revert(Error::OnlyOfferorCanRevoke);
    }
    if !token_market.is_active {
        runtime::revert(Error::OfferInactive)
    }

    token_market.is_active = false;
    token_market.offeror = None;
    // token_market.offeror;
    // When revoke-offer => token_market will be set is_active to false
    write_dictionary_value_from_key(SELLING_IN_MARKET, &token_id, token_market.clone());

    let token_ids: Vec<String> = vec![token_id];
    cep47_transfer_from(&contract_hash, get_self_key(), caller, token_ids);

    events::emit(&MarketPlaceEvent::Revoke {
        nft_contract: contract_hash,
        token_id: token_market.token_id.clone(),
        offeror: caller,
        minimum_offer: token_market.minimum_offer,
        is_active: false,
    });
}

#[no_mangle]
pub extern "C" fn set_support_token() {
    // let nft_contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);

    // let nft_contract_str_key = helpers::make_dictionary_item_key_for_key(nft_contract_hash);

    // // let current_contract_owner = runtime::get_key(CONTRACT_OWNER_KEY_NAME).unwrap_or_revert();

    // let caller = get_immediate_caller_key();

    // // let caller = helpers::get_verified_caller().unwrap_or_revert();
    // let current_contract_owner = helpers::get_stored_value_with_user_errors(
    //     CONTRACT_OWNER_KEY_NAME,
    //     Error::MissingContractOwner,
    //     Error::InvalidContractOwner,
    // );

    // if caller != current_contract_owner {
    //     runtime::revert(Error::InvalidContractOwner);
    // }
    // // Change from Vec<Key> to Vec<String>
    // let mut token_list = get_key::<Vec<String>>(TOKEN_CONTRACT_LIST).unwrap_or_revert();

    // let nft_enabled: bool = runtime::get_named_arg(ARG_NFT_ENABLED);
    // if nft_enabled {
    //     if !token_list.contains(&nft_contract_hash.to_string()) {
    //         token_list.push(nft_contract_hash.to_string());
    //         set_key(TOKEN_CONTRACT_LIST, token_list);
    //     }

    //     write_dictionary_value_from_key(TOKEN_CONTRACT_MAP, &nft_contract_str_key, true);
    // } else {
    //     if token_list.contains(&nft_contract_hash.to_string()) {
    //         token_list.retain(|x| *x != nft_contract_hash.to_string());
    //         set_key(TOKEN_CONTRACT_LIST, token_list);
    //     }
    //     write_dictionary_value_from_key(TOKEN_CONTRACT_MAP, &nft_contract_str_key, false);

    //     // write_dictionary_value_from_key(TOKEN_CONTRACT_MAP, &nft_contract_hash.to_string(), false);
    // }
}

#[no_mangle]
pub extern "C" fn transfer_owner() {
    let new_contract_owner: Key = runtime::get_named_arg(ARG_CONTRACT_OWNER);
    let current_contract_owner = helpers::get_stored_value_with_user_errors(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );

    let caller = get_immediate_caller_key();

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }
    set_key(CONTRACT_OWNER_KEY_NAME, new_contract_owner);
}

#[no_mangle]
pub extern "C" fn set_fee_receiver() {
    let fee_receiver: Key = runtime::get_named_arg(ARG_MARKET_FEE_RECEIVER);
    let current_contract_owner = helpers::get_stored_value_with_user_errors(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );
    let caller = get_immediate_caller_key();

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }
    set_key(MARKET_FEE_RECEIVER, fee_receiver);
}

#[no_mangle]
pub extern "C" fn emergency_withdraw_cspr() {
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);
    let current_contract_owner: Key = helpers::get_stored_value_with_user_errors(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );
    let caller = get_immediate_caller_key();

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }
    let contract_purse = helpers::get_uref(CONTRACT_PURSE);

    transfer_from_purse_to_account(
        contract_purse,
        caller.into_account().unwrap(),
        u256_to_u512(amount),
        None,
    )
    .unwrap_or_revert_with(Error::CanNotTransferCSPR);
}

#[no_mangle]
pub extern "C" fn emergency_withdraw_nfts() {
    let token_ids: Vec<String> = runtime::get_named_arg(ARG_TOKEN_IDS);
    let nft_contract_hash: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let current_contract_owner: Key = helpers::get_stored_value_with_user_errors(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );
    let caller = get_immediate_caller_key();

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }

    cep47_transfer_from(
        &nft_contract_hash,
        get_self_key(),
        current_contract_owner,
        token_ids,
    );
}

#[no_mangle]
pub extern "C" fn change_fee() {
    let caller = get_immediate_caller_key();
    let current_contract_owner = helpers::get_stored_value_with_user_errors(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }
    let new_fee: U256 = runtime::get_named_arg(MARKET_FEE);
    if new_fee > U256::from(200u64) {
        runtime::revert(Error::FeeTooHigh);
    }
    set_key(MARKET_FEE, new_fee);
}

fn set_selling(contract_hash: &Key, token_identifier: &String, offeror: Key, minimum_offer: U256) {
    let exists = get_dictionary_value_from_key::<SellingInMarket>(
        SELLING_IN_MARKET,
        &token_identifier.clone(),
    );
    let mut token_market = if let Some(..) = exists {
        exists.unwrap()
    } else {
        SellingInMarket {
            offeror: Some(offeror),
            token_id: token_identifier.to_string(),
            nft_contract: *contract_hash,
            minimum_offer,
            is_active: true,
            bidder: Vec::new(),
            bidding_price: Vec::new(),
        }
    };

    token_market.offeror = Some(offeror);
    token_market.is_active = true;
    token_market.minimum_offer = minimum_offer;

    if !token_market.bidding_price.is_empty()
        && token_market.minimum_offer <= *token_market.bidding_price.last().unwrap()
    {
        let contract_purse = helpers::get_uref(CONTRACT_PURSE);

        let bidder = *token_market.bidder.last().unwrap();
        let bidding_price = *token_market.bidding_price.last().unwrap();

        do_trade_change_price(
            &mut token_market,
            contract_purse,
            bidder,
            bidding_price,
            offeror,
        )
    } else {
        cep47_transfer_from(
            contract_hash,
            offeror,
            get_self_key(),
            [token_identifier.clone()].to_vec(),
        );

        write_dictionary_value_from_key(
            SELLING_IN_MARKET,
            &token_identifier.clone(),
            token_market.clone(),
        );

        events::emit(&MarketPlaceEvent::Sell {
            nft_contract: *contract_hash,
            token_id: token_identifier.to_string(),
            offeror,
            minimum_offer,
            is_active: true,
        });
    }
}

fn cep47_transfer_from(contract_hash: &Key, source: Key, target: Key, token_ids: Vec<String>) {
    let contract_hash_addr: HashAddr = contract_hash.into_hash().unwrap_or_revert();
    let contract_hash: ContractHash = ContractHash::new(contract_hash_addr);
    let _: () = runtime::call_contract(
        contract_hash,
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            "sender" => source,
            "recipient" => target,
            "token_ids" => token_ids
        },
    );
}

fn check_enabled_nft(contract_hash: Key) {
    let nft_contract_hash: Key = helpers::get_key(TOKEN_CONTRACT_SUPPORT).unwrap();

    if nft_contract_hash != contract_hash {
        runtime::revert(Error::UnsupportedToken);
    }
}
fn do_trade(
    selling_in_maket: &mut SellingInMarket,
    contract_purse: URef,
    bidder: Key,
    bidding_price: U256,
) {
    transfer_to_seller_and_fee(selling_in_maket, contract_purse, bidding_price);

    transfer_nfts_to_bidder(selling_in_maket, bidder, get_self_key());
    //dont touch the bidder list here, as the success bidder is already removed from the list

    selling_in_maket.is_active = false;
    let offeror = selling_in_maket.offeror.unwrap();
    selling_in_maket.offeror = None;
    write_dictionary_value_from_key(
        SELLING_IN_MARKET,
        &selling_in_maket.token_id.clone(),
        selling_in_maket.clone(),
    );
    events::emit(&MarketPlaceEvent::DoneSell {
        nft_contract: selling_in_maket.nft_contract,
        token_id: selling_in_maket.token_id.clone(),
        offeror,
        buyer: bidder,
        value: bidding_price,
    });
}

fn transfer_to_seller_and_fee(
    selling_in_maket: &SellingInMarket,
    contract_purse: URef,
    trade_price: U256,
) {
    let fee_portion: U256 = helpers::get_stored_value_with_user_errors(
        MARKET_FEE,
        Error::MissingFeePortion,
        Error::InvalidFeePortion,
    );
    let fee_amount_per_side = trade_price * fee_portion / U256::from(1000u64);

    let seller_receive_amount: U256 = trade_price - fee_amount_per_side;

    // transfer cspr to seller
    let seller_key: Key = selling_in_maket.offeror.unwrap();
    let seller_pubkey = seller_key.into_account().unwrap();

    transfer_from_purse_to_account(
        contract_purse,
        seller_pubkey,
        u256_to_u512(seller_receive_amount),
        None,
    )
    .unwrap_or_revert_with(Error::CanNotTransferCSPR);

    let total_fee: U256 = fee_amount_per_side;
    let fee_receiver: Key = helpers::get_stored_value_with_user_errors(
        MARKET_FEE_RECEIVER,
        Error::MissingFeeReceiver,
        Error::InvalidFeeReceiver,
    );

    transfer_from_purse_to_account(
        contract_purse,
        fee_receiver.into_account().unwrap(),
        u256_to_u512(total_fee),
        None,
    )
    .unwrap_or_revert_with(Error::CanNotTransferCSPR);
}

fn transfer_nfts_to_bidder(selling_in_maket: &SellingInMarket, bidder: Key, transfer_from: Key) {
    let token_ids = vec![selling_in_maket.token_id.clone()];
    cep47_transfer_from(
        &selling_in_maket.nft_contract,
        transfer_from,
        bidder,
        token_ids,
    );
}

fn do_trade_change_price(
    selling_in_maket: &mut SellingInMarket,
    contract_purse: URef,
    bidder: Key,
    bidding_price: U256,
    transfer_from: Key,
) {
    transfer_to_seller_and_fee(selling_in_maket, contract_purse, bidding_price);

    transfer_nfts_to_bidder(selling_in_maket, bidder, transfer_from);

    // remove the last bidder in the list here after success
    let last_index = selling_in_maket.bidder.len() - 1;
    selling_in_maket.minimum_offer = bidding_price;
    selling_in_maket.bidder.remove(last_index);
    selling_in_maket.bidding_price.remove(last_index);

    selling_in_maket.is_active = false;
    let offeror = selling_in_maket.offeror.unwrap();
    selling_in_maket.offeror = None;

    write_dictionary_value_from_key(
        SELLING_IN_MARKET,
        &selling_in_maket.token_id.clone(),
        selling_in_maket.clone(),
    );
    events::emit(&MarketPlaceEvent::DoneSell {
        nft_contract: selling_in_maket.nft_contract,
        token_id: selling_in_maket.token_id.clone(),
        offeror,
        buyer: bidder,
        value: bidding_price,
    });
}
fn insert_new_bidder(selling_in_maket: &mut SellingInMarket, bidder: Key, bidding_price: U256) {
    if !selling_in_maket.bidding_price.is_empty() {
        let idx = selling_in_maket
            .bidding_price
            .partition_point(|&x| x < bidding_price);
        selling_in_maket.bidding_price.insert(idx, bidding_price);
        selling_in_maket.bidder.insert(idx, bidder);
    } else {
        selling_in_maket.bidding_price.push(bidding_price);
        selling_in_maket.bidder.push(bidder);
    }
}
