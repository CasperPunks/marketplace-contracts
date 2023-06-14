use alloc::{
    string::{String, ToString},
    vec::*,
};

use casper_contract::contract_api::{storage, system};
use casper_types::{contracts::NamedKeys, ContractPackageHash, Key, URef, U256};

use crate::constants::*;
pub fn default(
    contract_owner: Key,
    _market_fee_receiver: Key,
    _market_fee: U256,
    contract_package_hash: ContractPackageHash,
    fee_token: Option<Key>,
) -> NamedKeys {
    let mut named_keys = NamedKeys::new();

    // Contract 'Named keys'
    named_keys.insert(
        CONTRACT_OWNER_KEY_NAME.to_string(),
        Key::from(storage::new_uref(contract_owner)),
    );
    // named_keys.insert(
    //     MARKET_FEE_RECEIVER.to_string(),
    //     Key::from(storage::new_uref(market_fee_receiver)),
    // );
    // named_keys.insert(
    //     MARKET_FEE.to_string(),
    //     Key::from(storage::new_uref(U256::zero())),
    // );
    named_keys.insert(
        TOKEN_CONTRACT_LIST.to_string(),
        Key::from(storage::new_uref(Vec::<String>::new())),
    );
    named_keys.insert(
        "contract_package_hash".to_string(),
        storage::new_uref(contract_package_hash).into(),
    );
    // create contract purse
    let contract_purse: URef = system::create_purse();
    named_keys.insert(CONTRACT_PURSE.to_string(), contract_purse.into());

    if let Some(..) = fee_token {
        named_keys.insert(
            FEE_TOKEN_KEY_NAME.to_string(),
            Key::from(storage::new_uref(fee_token.unwrap())),
        );
    }

    named_keys
}
