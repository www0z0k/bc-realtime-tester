#![allow(non_snake_case)]
mod game;

use crate::game::Hero;
use crate::game::Fillable;
use crate::game::Stats;
use crate::game::Trap;
use crate::game::BattleLog;
use crate::game::SingleBattleResult;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::BorshStorageKey;
use near_sdk::{env, log, near_bindgen, AccountId};
use serde::Serialize;
use std::collections::HashMap;
use near_sdk::collections::UnorderedMap;
use near_sdk::collections::LazyOption;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::convert::TryInto;

pub(crate) fn rand_range(from: u16, to: u16, start: usize) -> u16 {
    rand_range_from_seed(from, to, env::random_seed(), start)
}
pub(crate) fn rand_range_from_seed(from: u16, to: u16, seed: Vec<u8>, start: usize) -> u16 {
    let seed: [u8; 32] = rearrange(start, seed.try_into().unwrap());
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    rng.gen_range(from, to)
}

fn rearrange(breaking: usize, arr: [u8; 32]) -> [u8; 32] {
    let l = arr.len();
    let first_and_last = [&arr[breaking..l], &arr[..breaking]].concat();
    return first_and_last.try_into().unwrap();
}

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKeys {
    Accounts,
    Heroes,
    Balances,
    UsersHeroes,
    Globals,
    Traps,
    UsersTraps,
    UsersGold,
    Timings,
}



#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TribeTerra {
    records: UnorderedMap<AccountId, String>,
    heroes: UnorderedMap<u64, Hero>,
    traps: UnorderedMap<u64, Trap>,
    usersHeroes: UnorderedMap<AccountId, Vec<u64>>,
    usersTraps: UnorderedMap<AccountId, Vec<u64>>,
    globals: UnorderedMap<String, u64>,
    timings: UnorderedMap<String, u64>,
    usersGold: UnorderedMap<AccountId, f64>,
}

impl Default for TribeTerra {
    fn default() -> Self {
        Self {
            records: UnorderedMap::new(StorageKeys::Accounts),
            heroes: UnorderedMap::new(StorageKeys::Heroes),
            usersHeroes: UnorderedMap::new(StorageKeys::UsersHeroes),
            traps: UnorderedMap::new(StorageKeys::Traps),
            usersTraps: UnorderedMap::new(StorageKeys::UsersTraps),
            globals: UnorderedMap::new(StorageKeys::Globals),
            timings: UnorderedMap::new(StorageKeys::Timings),
            usersGold: UnorderedMap::new(StorageKeys::UsersGold),
        }
    }
}

#[near_bindgen]
impl TribeTerra {

    pub fn set_interval(&mut self, id: String) -> u64 {
        if self.timings.get(&id) != std::option::Option::None {
            return 0; // interval already set
        }

        self.timings.insert(&id, &env::block_timestamp());
        return 1;
    }

    pub fn get_interval(&self, id: String) -> u64 {
        if self.timings.get(&id) == std::option::Option::None {
            return 0; // unknown interval 
        }

        return env::block_timestamp() - self.timings.get(&id).unwrap();
    }
    

    pub fn get_time(&self) -> u64 {
        env::block_timestamp()
    }

    pub fn account_inited(&self, account_id: String) -> bool {
        return self.records.get(&account_id) != std::option::Option::None
    }

    pub fn get_tokens(&self, account_id: String) -> Option<String> {
        return self.records.get(&account_id);
    }
    
    pub fn init(&mut self) {
        let account_id = env::signer_account_id();
        self.records.insert(&account_id, &"inited".to_owned());

        let h1 = self.create_hero("common".to_string(), 6, 1);
        self.heroes.insert(&h1.id, &h1);
        let h2 = self.create_hero("common".to_string(), 6, 4);
        self.heroes.insert(&h2.id, &h2);
        let h3 = self.create_hero("common".to_string(), 6, 7);
        self.heroes.insert(&h3.id, &h3);

        let t1 = self.create_trap(10, 7, 6, 1);
        self.traps.insert(&t1.id, &t1);
        let t2 = self.create_trap(10, 7, 6, 2);
        self.traps.insert(&t2.id, &t2);
        let t3 = self.create_trap(10, 7, 6, 3);
        self.traps.insert(&t3.id, &t3);
        
        let mut opt:Vec<u64> = vec![h1.id, h2.id, h3.id];
        self.usersHeroes.insert(&account_id, &opt);

        self.usersGold.insert(&account_id, &1000000.0);

        let mut optTraps:Vec<u64> = vec![t1.id, t2.id, t3.id];
        self.usersTraps.insert(&account_id, &optTraps);
    }

    pub fn add_token(&mut self, message: String) {
        let account_id = env::signer_account_id();
        if self.records.get(&account_id) != std::option::Option::None {
            let newstr = self.records.get(&account_id).unwrap() + &"|".to_owned() + &message;
            self.records.insert(&account_id, &newstr);
        } else {
            self.records.insert(&account_id, &message);
        }
    }

    pub fn list_all_users(self) -> Vec<String> {
        let keys = self.records.keys_as_vector();
        (0..self.records.len())
            .map(|index| (keys.get(index).unwrap()))
            .collect()
    }

    fn transfer_gold_from_to(&mut self, sender: String, target: String, amount: f64) -> f64 {
        let mut sGold = self.usersGold.get(&sender).unwrap();
        let mut tGold = self.usersGold.get(&target).unwrap();
        if sGold < amount || sender == target {
            return 0.0
        }
        sGold -= amount;
        tGold += amount;
        self.usersGold.insert(&sender, &sGold);
        self.usersGold.insert(&target, &tGold);
        return amount
    }

    pub fn transfer_gold_to(&mut self, target: String, amount: u64) -> f64 {
        let sender = env::signer_account_id();
        return self.transfer_gold_from_to(sender, target, amount as f64)
    }

    pub fn get_user_gold(&self, account_id: String) -> Option<f64> {
        return self.usersGold.get(&account_id);
    }

    pub fn get_user_heroes(&self, account_id: String) -> Option<Vec<u64>> {
        return self.usersHeroes.get(&account_id);
    }

    pub fn hero_by_id(&self, id: u64) -> Option<Hero> {
        return self.heroes.get(&id);
    }

    pub fn add_to_stat(&mut self, id: u64, stat: String) -> Option<Hero> {
        let sender = env::signer_account_id();
        let stats = vec!["vitality", "strength", "agility", "intelligence"];
        if self.get_user_heroes(sender.to_owned()).unwrap().contains(&id) && stats.contains(&&*stat) {
            let mut currHero = self.hero_by_id(id).unwrap();
            currHero.improveStat(stat);
            self.heroes.insert(&currHero.id, &currHero);
            return Some(currHero);
        }
        return None;
    }

    pub fn get_user_traps(&self, account_id: String) -> Option<Vec<u64>> {
        return self.usersTraps.get(&account_id);
    }

    pub fn trap_by_id(&self, id: u64) -> Option<Trap> {
        return self.traps.get(&id);
    }

    fn create_hero(&mut self, rarity: String, diceVal: u16, seedOffset: usize) -> Hero {
        if self.globals.get(&"lastHeroID".to_owned()) == std::option::Option::None {
            self.globals.insert(&"lastHeroID".to_owned(), &0);
        }

        let seedPoints = if rarity == "common" { rand_range(5, 10, seedOffset) } else { rand_range(11, 20, seedOffset) };
        let newID = self.globals.get(&"lastHeroID".to_owned()).unwrap() + 1;
        self.globals.insert(&"lastHeroID".to_owned(), &newID);
        let offset = seedOffset * 3;
        return Hero::new(seedPoints, 1, diceVal, newID, offset); // no more than 3 in one block
    }

    fn create_trap(&mut self, power: u16, value: u16, diceVal: u16, seedOffset: usize) -> Trap {
        if self.globals.get(&"lastHeroID".to_owned()) == std::option::Option::None {
            self.globals.insert(&"lastHeroID".to_owned(), &0);
        }

        let newID = self.globals.get(&"lastHeroID".to_owned()).unwrap() + 1;
        self.globals.insert(&"lastHeroID".to_owned(), &newID);
        let stat = rand_range(0, 3, seedOffset);
        let offset = seedOffset * 3;
        return Trap::new(power, value, diceVal, stat, newID, offset);  // no more than 3 in one block
    }

    pub fn do_dungeon(&mut self, defender: String) -> BattleLog {
        let attacker = env::signer_account_id();

        let mut res = BattleLog {
            firstTrap: Vec::new(),
            secondTrap: Vec::new(),
            thirdTrap: Vec::new(),
            rewardGold: 0.0,
        };
        if !self.account_inited(attacker.to_owned()) || !self.account_inited(defender.to_owned()) {
            return res
        }

        let mut seed = 1;
        let mut nTrap = 1;
        let mut deadHeroes = 0;
        let mut deadTraps = 0;
        let mut failed = false;
        
        for trap in self.get_user_traps(defender.to_owned()).unwrap().into_iter() {
            let mut currTrap = self.trap_by_id(trap).unwrap();
            let mut hitHeroes = 0;
            for hero in self.get_user_heroes(attacker.to_owned()).unwrap().into_iter() {
                let mut currHero = self.hero_by_id(hero).unwrap();
                if !currTrap.enabled || currHero.health.current == 0 || currTrap.health.current == 0 {
                    if currHero.health.current == 0 {
                        hitHeroes += 1;
                    }
                    continue;
                }
                let damages = currHero.rollStatAgainst(currTrap.stat, currTrap.value, seed);
                currTrap.takeDamage(damages.1);
                seed += 1;
                let attackRes = SingleBattleResult {
                    heroID: currHero.id,
                    trapID: currTrap.id,
                    damageToTrap: damages.1,
                    damageToHero: damages.0,
                    heroHP: currHero.health.current,
                    trapHP: currTrap.health.current,
                };

                if damages.0 > 0 {
                    hitHeroes += 1;
                }
                
                if nTrap == 1 { res.firstTrap.push(attackRes) } else if nTrap == 2 { res.secondTrap.push(attackRes) } else if nTrap == 3 { res.thirdTrap.push(attackRes) } 

                if currHero.health.current == 0 {
                    deadHeroes += 1;
                }

                if currTrap.health.current == 0 {
                    deadTraps += 1;
                }
                // save - this way looks ugly, can I do it by reference?
                self.heroes.insert(&currHero.id, &currHero);
            }
            if hitHeroes == 3 {
                currTrap.enabled = true;
                failed = true;
                self.traps.insert(&currTrap.id, &currTrap);
                break;
            }
            
            if !currTrap.enabled || deadHeroes > 2 {
                nTrap += 1;
                currTrap.enabled = true;
            }
            self.traps.insert(&currTrap.id, &currTrap);
        }
        
        let basicReward = self.get_user_gold(attacker.to_owned()).unwrap();
        if deadHeroes < 3 && !failed {
            res.rewardGold = basicReward * 0.1 * (1.0 + (3 - deadHeroes) as f64 * 0.01 + deadTraps as f64 * 0.01);
            self.transfer_gold_from_to(defender.to_owned(), attacker.to_owned(), res.rewardGold);
        }
        return res
    }

    pub fn test1hero(&self) -> Hero {
        return Hero::new(123, 12, 6, 1, 0)
    }

    pub fn test1trap(&self) -> Trap {
        return Trap::new(10, 7, 6, 0, 1, 0)
    }
}

