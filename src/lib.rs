#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_must_use)]

/*
cargo build --all --target wasm32-unknown-unknown --release && cp target/wasm32-unknown-unknown/release/near_speedtest.wasm res/near_speedtest.wasm && near dev-deploy res/near_speedtest.wasm
near view dev-1660092891396-92529334966693 get_all
near call dev-1660092891396-92529334966693 move_char '{"dx": 1, "dy": 0}' --accountId 'www0rker.testnet'
*/

mod grid;
use crate::grid::Char;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::BorshStorageKey;
use near_sdk::test_utils::accounts;
use near_sdk::{env, log, near_bindgen, AccountId};
use serde::Serialize;
use std::collections::HashMap;
use near_sdk::collections::UnorderedMap;
use near_sdk::collections::LazyOption;
use near_sdk::collections::Vector;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::convert::TryInto;

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKeys {
    Accounts,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Speedtest {
    coords: UnorderedMap<AccountId, Char>,
}

impl Default for Speedtest {
    fn default() -> Self {
        Self {
            coords: UnorderedMap::new(StorageKeys::Accounts),
        }
    }
}

#[near_bindgen]
impl Speedtest {
    pub fn get_all(&self) -> Vec<Char> {
        let keys = self.coords.keys_as_vector();
        return keys.iter().map(|key| self.coords.get(&key).unwrap()).collect();
    }

    pub fn reset_position(&mut self) -> Vec<Char> {
        let account_id = env::signer_account_id();
        let mut char = self.coords.get(&account_id).unwrap_or(Char::new(account_id.to_owned(), 0, 0));
        char.x = 0;
        char.y = 0;
        self.coords.insert(&account_id, &char);
        return self.get_all();
    }
    pub fn move_char(&mut self, dx: i32, dy: i32) -> Vec<Char> {
        let account_id = env::signer_account_id();
        if ((dx == 1 || dx == -1) && dy == 0) || (dx == 0 && (dy == 1 || dy == -1)) {
            let mut char = self.coords.get(&account_id).unwrap_or(Char::new(account_id.to_owned(), 0, 0));
            char.x += dx;
            char.y += dy;
            self.coords.insert(&account_id, &char);
        }
        return self.get_all();
    }
}
