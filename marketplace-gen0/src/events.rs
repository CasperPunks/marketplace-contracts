#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// use std::collections::BTreeMap;

extern crate alloc;
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::*,
};

use casper_contract::contract_api::storage;
use casper_types::{ContractPackageHash, Key, URef, U256};

use crate::helpers::*;

pub enum MarketPlaceEvent {
    Sell {
        nft_contract: Key,
        token_id: String,
        offeror: Key,
        minimum_offer: U256,
        is_active: bool,
    },
    Revoke {
        nft_contract: Key,
        token_id: String,
        offeror: Key,
        minimum_offer: U256,
        is_active: bool,
    },
    DoneSell {
        nft_contract: Key,
        token_id: String,
        offeror: Key,
        buyer: Key,
        value: U256,
    },
    ChangePrice {
        nft_contract: Key,
        token_id: String,
        offeror: Key,
        new_price: U256,
        is_active: bool,
    },
    Bid {
        nft_contract: Key,
        token_id: String,
        offeror: Key,
        bidder: Key,
        value: U256,
    },
    RevokeBid {
        nft_contract: Key,
        token_id: String,
        offeror: Key,
        bidder: Key,
        value: U256,
        is_revoke: bool,
    },
}

impl MarketPlaceEvent {
    pub fn type_name(&self) -> String {
        match self {
            MarketPlaceEvent::Sell {
                nft_contract: _,
                token_id: _,
                offeror: _,
                minimum_offer: _,
                is_active: _,
            } => "sell",
            MarketPlaceEvent::ChangePrice {
                nft_contract: _,
                token_id: _,
                offeror: _,
                new_price: _,
                is_active: _,
            } => "change_price",
            MarketPlaceEvent::Revoke {
                nft_contract: _,
                token_id: _,
                offeror: _,
                minimum_offer: _,
                is_active: _,
            } => "revoke",
            MarketPlaceEvent::DoneSell {
                nft_contract: _,
                token_id: _,
                offeror: _,
                buyer: _,
                value: _,
            } => "donesell",
            MarketPlaceEvent::Bid {
                nft_contract: _,
                token_id: _,
                offeror: _,
                bidder: _,
                value: _,
            } => "bid",
            MarketPlaceEvent::RevokeBid {
                nft_contract: _,
                token_id: _,
                offeror: _,
                bidder: _,
                value: _,
                is_revoke: _,
            } => "revokebid",
        }
        .to_string()
    }
}

pub fn contract_package_hash() -> ContractPackageHash {
    get_key::<ContractPackageHash>("contract_package_hash").unwrap()
}

// pub(crate) fn contract_package_hash() -> ContractPackageHash {
//     let key : Key = runtime::get_key("contract_package_hash").unwrap();
//     let contract_package_hash_addr: HashAddr = key.into_hash().unwrap();
//     let factory_package_hash: ContractPackageHash = ContractPackageHash::new(contract_package_hash_addr);
//     factory_package_hash

// }

pub(crate) fn emit(pair_event: &MarketPlaceEvent) {
    let mut events = Vec::new();
    // let package : ContractPackageHash = runtime::get_key("contract_package_hash").into_hash().unwrap_or_revert();
    let package = contract_package_hash();
    match pair_event {
        MarketPlaceEvent::Sell {
            nft_contract,
            token_id,
            offeror,
            minimum_offer,
            is_active,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("nft_contract", nft_contract.to_string());
            event.insert("token_id", token_id.to_string());
            event.insert("offeror", offeror.to_string());
            event.insert("minimum_offer", minimum_offer.to_string());
            event.insert("is_active", is_active.to_string());
            events.push(event);
        }

        MarketPlaceEvent::ChangePrice {
            nft_contract,
            token_id,
            offeror,
            new_price,
            is_active,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("nft_contract", nft_contract.to_string());
            event.insert("token_id", token_id.to_string());
            event.insert("offeror", offeror.to_string());
            event.insert("new_price", new_price.to_string());
            event.insert("is_active", is_active.to_string());
            events.push(event);
        }

        MarketPlaceEvent::Revoke {
            nft_contract,
            token_id,
            offeror,
            minimum_offer,
            is_active,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("nft_contract", nft_contract.to_string());
            event.insert("token_id", token_id.to_string());
            event.insert("offeror", offeror.to_string());
            event.insert("minimum_offer", minimum_offer.to_string());
            event.insert("is_active", is_active.to_string());
            events.push(event);
        }

        MarketPlaceEvent::DoneSell {
            nft_contract,
            token_id,
            offeror,
            buyer,
            value,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("nft_contract", nft_contract.to_string());
            event.insert("token_id", token_id.to_string());
            event.insert("offeror", offeror.to_string());
            event.insert("buyer", buyer.to_string());
            event.insert("value", value.to_string());
            events.push(event);
        }

        MarketPlaceEvent::Bid {
            nft_contract,
            token_id,
            offeror,
            bidder,
            value,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("nft_contract", nft_contract.to_string());
            event.insert("token_id", token_id.to_string());
            event.insert("offeror", offeror.to_string());
            event.insert("bidder", bidder.to_string());
            event.insert("value", value.to_string());
            events.push(event);
        }

        MarketPlaceEvent::RevokeBid {
            nft_contract,
            token_id,
            offeror,
            bidder,
            value,
            is_revoke,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("nft_contract", nft_contract.to_string());
            event.insert("token_id", token_id.to_string());
            event.insert("offeror", offeror.to_string());
            event.insert("bidder", bidder.to_string());
            event.insert("value", value.to_string());
            event.insert("is_revoke", is_revoke.to_string());
            events.push(event);
        }
    };
    for event in events {
        let _: URef = storage::new_uref(event);
    }
}
