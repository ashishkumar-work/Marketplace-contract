//! # Marketplace Contract
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error,
    storage::Persistent,
    token::{self, Client},
    Address, Env, Map, Symbol,
};

#[derive(Clone, Copy)]
#[contracttype]
enum DataKey {
    Admin = 1,
    Initialized = 2,
    Assets = 3,
    Token = 4,
    LastID = 5,
}


#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    
    AlreadyInitialized = 1,
    InvalidAssetPrice = 2,
    BalanceTooLow = 3,
    AssetNotListed = 4,
    NotInitialized = 6,
    InvalidQuantity = 7,
}

#[contracttype]
#[derive(Clone)]
pub struct Asset {
    id: u64,
    asset_address: Address,
    owner: Address,
    price: i128,
    quantity: i128,
    listed: bool,
}

type AssetStorage = Map<u64, Asset>;

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    pub fn init(env: Env, token: Address, admin: Address) {
        admin.require_auth();
        let storage = env.storage().persistent();

        if storage.get::<_, ()>(&DataKey::Initialized).is_some() {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }

        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::Initialized, &());
        storage.set(&DataKey::Assets, &Map::<u64, Asset>::new(&env));
        storage.set(&DataKey::LastID, &1u64);
    }

    pub fn register(_: Env, trader: Address) {
        trader.require_auth();
    }

    pub fn get_listing(env: Env, id: u64) -> Option<Asset> {
        let storage = env.storage().persistent();
        Self::must_be_initialized(&env, &storage);
        let assets: AssetStorage = storage.get(&DataKey::Assets).unwrap();
        assets.get(id)
    }


    pub fn create_listing(
        env: Env,
        seller: Address,
        asset: Address,
        price: i128,
        quantity: i128,
    ) -> u64 {
        seller.require_auth();
        if price <= 0 {
            panic_with_error!(&env, Error::InvalidAssetPrice);
        }
        if quantity <= 0 {
            panic_with_error!(&env, Error::InvalidQuantity);
        }
        let storage = env.storage().persistent();

        Self::must_be_initialized(&env, &storage);

        let mut assets: AssetStorage = storage.get(&DataKey::Assets).unwrap();
        let id = Self::current_id(&storage);
        assets.set(
            id,
            Asset {
                id,
                asset_address: asset.clone(),
                owner: seller.clone(),
                price,
                quantity,
                listed: true,
            },
        );
        storage.set(&DataKey::Assets, &assets);

        let asset_client = Client::new(&env, &asset);

        if asset_client.balance(&seller) < quantity {
            panic_with_error!(&env, Error::BalanceTooLow);
        }

        asset_client.transfer(&seller, &env.current_contract_address(), &quantity);

        let topics = (Symbol::new(&env, "create_listing"), (seller));
        env.events().publish(topics, id);

        id
    }

    fn must_be_initialized(env: &Env, storage: &Persistent) {
        if storage.get::<_, ()>(&DataKey::Initialized).is_none() {
            panic_with_error!(&env, Error::NotInitialized);
        }
    }

    fn current_id(storage: &Persistent) -> u64 {
        let id: u64 = storage.get(&DataKey::LastID).unwrap();
        storage.set(&DataKey::LastID, &id.checked_add(1).unwrap());
        id
    }

    pub fn buy_listing(env: Env, buyer: Address, id: u64) {
        buyer.require_auth();
        let storage = env.storage().persistent();

        Self::must_be_initialized(&env, &storage);

        let token = storage.get(&DataKey::Token).unwrap();
        let mut assets: AssetStorage = storage.get(&DataKey::Assets).unwrap();
        let Asset {
            id,
            asset_address,
            owner: seller,
            price,
            quantity,
            listed,
        } = assets.get(id).unwrap();

        if !listed {
            panic_with_error!(&env, Error::AssetNotListed);
        }

        let token = token::Client::new(&env, &token);
        if token.balance(&buyer) < price * quantity {
            panic_with_error!(&env, Error::BalanceTooLow);
        }
        token.transfer(&buyer, &seller, &(price * quantity));
        assets.remove(id);
        storage.set(&DataKey::Assets, &assets);

        let asset_client = Client::new(&env, &asset_address);
        asset_client.transfer(&env.current_contract_address(), &buyer, &quantity);

        let topics = (Symbol::new(&env, "buy_listing"), (buyer));
        env.events().publish(topics, id);
    }

    pub fn update_price(env: Env, id: u64, new_price: i128) {
        if new_price <= 0 {
            panic_with_error!(&env, Error::InvalidAssetPrice);
        }

        let storage = env.storage().persistent();

        Self::must_be_initialized(&env, &storage);

        let mut assets: AssetStorage = storage.get(&DataKey::Assets).unwrap();
        let Asset {
            id,
            asset_address,
            owner: seller,
            quantity,
            listed,
            ..
        } = assets.get(id).unwrap();

        seller.require_auth();

        assets.set(
            id,
            Asset {
                id,
                asset_address,
                owner: seller.clone(),
                price: new_price,
                quantity,
                listed,
            },
        );
        storage.set(&DataKey::Assets, &assets);
        let topics = (Symbol::new(&env, "update_price"), (seller));
        env.events().publish(topics, id);
    }

    pub fn pause_listing(env: Env, id: u64) {
        let storage = env.storage().persistent();

        Self::must_be_initialized(&env, &storage);

        let mut assets: AssetStorage = storage.get(&DataKey::Assets).unwrap();
        let Asset {
            asset_address,
            owner,
            price,
            quantity,
            ..
        } = assets.get(id).unwrap();

        owner.require_auth();

        assets.set(
            id,
            Asset {
                id,
                asset_address,
                owner: owner.clone(),
                price,
                quantity,
                listed: false,
            },
        );
        storage.set(&DataKey::Assets, &assets);
        let topics = (Symbol::new(&env, "pause_listing"), (owner));
        env.events().publish(topics, id);
    }

    pub fn unpause_listing(env: Env, id: u64) {
        let storage = env.storage().persistent();

        Self::must_be_initialized(&env, &storage);

        let mut assets: AssetStorage = storage.get(&DataKey::Assets).unwrap();
        let Asset {
            id,
            asset_address,
            owner,
            price,
            quantity,
            ..
        } = assets.get(id).unwrap();

        owner.require_auth();

        assets.set(
            id,
            Asset {
                id,
                asset_address,
                owner: owner.clone(),
                price,
                quantity,
                listed: true,
            },
        );
        storage.set(&DataKey::Assets, &assets);
        let topics = (Symbol::new(&env, "unpause_listing"), (owner));
        env.events().publish(topics, id);
    }
    pub fn remove_listing(env: Env, id: u64) {
        let storage = env.storage().persistent();

        Self::must_be_initialized(&env, &storage);

        let mut assets: AssetStorage = storage.get(&DataKey::Assets).unwrap();
        let Asset {
            owner,
            asset_address,
            quantity,
            ..
        } = assets.get(id).unwrap();

        owner.require_auth();

        assets.remove(id).unwrap();
        storage.set(&DataKey::Assets, &assets);

        let asset_client = Client::new(&env, &asset_address);
        asset_client.transfer(&env.current_contract_address(), &owner, &quantity);

        let topics = (Symbol::new(&env, "remove_listing"), (owner));
        env.events().publish(topics, id);
    }
}

#[cfg(test)]
mod test;
