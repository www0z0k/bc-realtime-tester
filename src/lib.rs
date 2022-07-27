#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_must_use)]

mod game;
mod coins;

use crate::game::Hero;
use crate::game::Fillable;
use crate::game::Stats;
use crate::game::Trap;
use crate::game::BattleLog;
use crate::game::SingleBattleResult;
use crate::game::AttackLogRecord;
use crate::game::BASE_POWER;
use crate::game::POINT_PRICE;
use crate::coins::Coins;
use crate::coins::FEE;
use crate::coins::LEAGUES;
use crate::coins::League;

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
    UsersDungeonGold,
    Timings,
    Fighters,
    Dungeons,
    AttackLog,
    HeroesForUser,
    TrapsForUser,
}



#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TribeTerra {
    records: UnorderedMap<AccountId, String>,
    heroes: UnorderedMap<u64, Hero>,
    traps: UnorderedMap<u64, Trap>,
    usersHeroes: UnorderedMap<AccountId, Vec<u64>>,
    usersTraps: UnorderedMap<AccountId, Vec<u64>>,
    timings: UnorderedMap<String, u64>,
    fighters: UnorderedMap<AccountId, String>,
    dungeons: UnorderedMap<AccountId, Vec<u64>>,
    attackLog: UnorderedMap<AccountId, Vec<AttackLogRecord>>,
    coins: Coins,
    heroesForUser: UnorderedMap<AccountId, Vec<Hero>>,
    trapsForUser: UnorderedMap<AccountId, Vec<Trap>>,

}

impl Default for TribeTerra {
    fn default() -> Self {
        Self {
            records: UnorderedMap::new(StorageKeys::Accounts),
            heroes: UnorderedMap::new(StorageKeys::Heroes),
            usersHeroes: UnorderedMap::new(StorageKeys::UsersHeroes),
            traps: UnorderedMap::new(StorageKeys::Traps),
            usersTraps: UnorderedMap::new(StorageKeys::UsersTraps),
            timings: UnorderedMap::new(StorageKeys::Timings),
            fighters: UnorderedMap::new(StorageKeys::Fighters),
            dungeons: UnorderedMap::new(StorageKeys::Dungeons),
            attackLog: UnorderedMap::new(StorageKeys::AttackLog),
            heroesForUser: UnorderedMap::new(StorageKeys::HeroesForUser),
            trapsForUser: UnorderedMap::new(StorageKeys::TrapsForUser),
            coins: Coins::new(),
        }
    }
}

#[near_bindgen]
impl TribeTerra {

    pub fn list_leagues(self) -> Vec<League> {
        return LEAGUES.to_vec();
    }

    // TODO apply fee here
    pub fn open_dungeon(&mut self, index: usize, trap_0: Option<u64>, trap_1: Option<u64>, trap_2: Option<u64>) -> u8 {
        if trap_0 != None && trap_1 != None && trap_2 != None {
            // setup dungeon here
            let is_set = self.setup_traps(trap_0, trap_1, trap_2);
            if !is_set {
                return 0; // something went wrong
            }
        }

        let account_id = env::signer_account_id();
        let mut key = account_id.to_owned();
        key.push_str(&"-dungeon");
        if self.set_interval(key) == 0 {
            return 0; // already open
        }
        self.coins.fill_dungeon_stash(LEAGUES[index].stash);
        let mut tier_id = String::from("tier_");
        tier_id.push_str(&index.to_string().to_owned());
        self.fighters.insert(&account_id, &tier_id);
        return 1;
    }

    pub fn setup_traps(&mut self, trap_0: Option<u64>, trap_1: Option<u64>, trap_2: Option<u64>) -> bool {
        let account_id = env::signer_account_id();
        if self.usersTraps.get(&account_id) == None || trap_0 == None || trap_1 == None || trap_2 == None {
            return false;
        }
        let user_traps = self.usersTraps.get(&account_id).unwrap();
        if !user_traps.contains(&trap_0.unwrap()) || !user_traps.contains(&trap_1.unwrap()) || !user_traps.contains(&trap_2.unwrap()) {
            return false;
        }
        let input_traps = vec![trap_0.unwrap(), trap_1.unwrap(), trap_2.unwrap()];
        let mut v = vec![trap_0.unwrap(), trap_1.unwrap(), trap_2.unwrap()];
        v.sort_unstable();
        v.dedup();
        if input_traps.len() != v.len() {
            return false;
        }
        self.dungeons.insert(&account_id, &input_traps);
        return true;
    }

    pub fn get_user_tier(self, uid: String) -> (Option<String>, u64) {
        let tkey = format!("{}-dungeon", &uid.to_owned());
        return (self.fighters.get(&uid), self.timings.get(&tkey).unwrap_or(0));
    }

    pub fn list_fighters(&mut self, index: usize) -> Vec<String> {
        let mut tier_id = String::from("tier_");
        tier_id.push_str(&index.to_string().to_owned());

        let keys = self.fighters.keys_as_vector();
        let vec_keys: Vec<String> = (0..self.fighters.len())
            .map(|index| (keys.get(index).unwrap()))
            .collect();

        vec_keys
            .into_iter()
            .filter(|k| {
                if self.fighters.get(&k).unwrap() == tier_id {
                    // check time
                    let tkey = format!("{}-dungeon", &k.to_owned());
                    if LEAGUES[index as usize].duration > self.get_interval(tkey.to_owned()) {
                        return true;
                    } else {
                        // try cleanup
                        self.clear_interval(&tkey.to_owned());
                        self.fighters.remove(&k.to_owned());
                        return false;
                    }
                } 
                // another league
                return false;
            })
            .collect::<Vec<_>>()     
    }

    pub fn clear_interval(&mut self, id: &String) {
        self.timings.remove(&id);
    }

    pub fn set_interval(&mut self, id: String) -> u64 {
        if self.timings.get(&id) != None {
            return 0; // interval already set
        }
        let ts = env::block_timestamp();
        self.timings.insert(&id, &ts);
        return 1;
    }

    pub fn get_interval(&self, id: String) -> u64 {
        if self.timings.get(&id) == None {
            return 0; // unknown interval 
        }

        return env::block_timestamp() - self.timings.get(&id).unwrap();
    }

    pub fn get_time(&self) -> u64 {
        env::block_timestamp()
    }

    pub fn account_battles(&self, account_id: String) -> Option<Vec<AttackLogRecord>> {
        return self.attackLog.get(&account_id);
    }

    pub fn account_inited(&self, account_id: String) -> bool {
        return self.records.get(&account_id) != None;
    }

    pub fn get_tokens(&self, account_id: String) -> Option<String> {
        return self.records.get(&account_id);
    }
    
    pub fn init_contract(&mut self) {
        if self.coins.globals.get(&"systemInitialGold".to_owned()) == None {
            self.coins.globals.insert(&"systemInitialGold".to_owned(), &1000000000.0);
        }
        if self.coins.globals.get(&"systemGold".to_owned()) == None {
            self.coins.globals.insert(&"systemGold".to_owned(), &0.0);
        }
    }

    pub fn init(&mut self) {
        let account_id = env::signer_account_id();
        // TODO kill second init)
        self.records.insert(&account_id, &"inited".to_owned());

        self.coins.transfer_gold_from_system(account_id.to_owned(), 1000.0);
    }

    pub fn test_init(&mut self) {
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
        
        let opt:Vec<u64> = vec![h1.id, h2.id, h3.id];
        self.usersHeroes.insert(&account_id, &opt); 
        let optTraps:Vec<u64> = vec![t1.id, t2.id, t3.id];
        self.usersTraps.insert(&account_id, &optTraps);
        
        self.coins.transfer_gold_from_system(account_id.to_owned(), 1000.0);
    }

    pub fn gen_new_heroes_for_user(&mut self) {
        let account_id = env::signer_account_id();
        let seed = self.get_best_user_hero_power(account_id.to_owned());
        let h1 = self.create_hero_on_power(seed, 6, 1);
        let h2 = self.create_hero_on_power(seed, 6, 2);
        let h3 = self.create_hero_on_power(seed, 6, 3);
        let opt:Vec<Hero> = vec![h1, h2, h3];
        self.heroesForUser.insert(&account_id, &opt); 
    }
    
    pub fn get_tavern_heroes(&self, account_id: String) -> Option<Vec<Hero>> {
        return self.heroesForUser.get(&account_id);
    }

    pub fn get_point_price(&self) -> f64 {
        return POINT_PRICE;
    }

    pub fn hire_hero(&mut self, index: usize) -> Option<Vec<Hero>> {
        let account_id = env::signer_account_id();
        let mut heroes = self.heroesForUser.get(&account_id).unwrap();

        let power = heroes[index].totalPower();
        let price = power as f64 * POINT_PRICE;
        if price > self.coins.get_user_gold(account_id.to_owned()).unwrap_or(0.0) {
            return self.heroesForUser.get(&account_id);
        }

        self.coins.transfer_gold_to_system(account_id.to_owned(), price);

        let mut heroToMint = heroes.remove(index);
        heroToMint.id = self.coins.next_id();

        self.heroes.insert(&heroToMint.id, &heroToMint);

        let mut userHeroIDs = self.usersHeroes.get(&account_id).unwrap_or(Vec::<u64>::new());
        userHeroIDs.push(heroToMint.id);
        self.usersHeroes.insert(&account_id, &userHeroIDs);

        self.heroesForUser.insert(&account_id, &heroes);
        return self.heroesForUser.get(&account_id);
    }

    pub fn add_token(&mut self, message: String) {
        let account_id = env::signer_account_id();
        if self.records.get(&account_id) != None {
            let newstr = self.records.get(&account_id).unwrap() + &"|".to_owned() + &message;
            self.records.insert(&account_id, &newstr);
        } else {
            self.records.insert(&account_id, &message);
        }
    }

    pub fn get_user_money(self, account_id: String) -> Vec<Option<f64>> {
        vec![self.coins.get_user_gold(account_id.to_owned()), self.coins.get_user_dungeon_gold(account_id.to_owned())]
    }

    pub fn list_all_users(self) -> Vec<String> {
        let keys = self.records.keys_as_vector();
        (0..self.records.len())
            .map(|index| (keys.get(index).unwrap()))
            .collect()
    }

    pub fn get_user_heroes(&self, account_id: String) -> Option<Vec<u64>> {
        return self.usersHeroes.get(&account_id);
    }

    pub fn get_best_user_hero_power(&self, account_id: String) -> u16 {
        let heroids = self.usersHeroes.get(&account_id);
        if heroids == None {
            return BASE_POWER;
        }
        let powers: Vec<u16> = heroids.unwrap().into_iter().map(|index| self.hero_by_id(index).unwrap().totalPower()).collect();
        if powers.len() == 0 {
            return BASE_POWER;
        }
        return powers.iter().max().unwrap().to_owned();
    }

    pub fn get_best_user_trap_power(&self, account_id: String) -> u16 {
        let trapids = self.usersTraps.get(&account_id);
        if trapids == None {
            return BASE_POWER;
        }
        let powers: Vec<u16> = trapids.unwrap().into_iter().map(|index| self.trap_by_id(index).unwrap().power).collect();
        if powers.len() == 0 {
            return BASE_POWER;
        }
        return powers.iter().max().unwrap().to_owned();
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

    pub fn get_dungeon(&self, account_id: String) -> Option<Vec<u64>> {
        return self.dungeons.get(&account_id);
    }

    pub fn trap_by_id(&self, id: u64) -> Option<Trap> {
        return self.traps.get(&id);
    }

    fn create_hero(&mut self, rarity: String, diceVal: u16, seedOffset: usize) -> Hero {
        let seedPoints = if rarity == "common" { rand_range(5, 10, seedOffset) } else { rand_range(11, 20, seedOffset) };
        let newID = self.coins.next_id();
        let offset = seedOffset * 3;
        return Hero::new(seedPoints, 1, diceVal, newID, offset); // no more than 3 in one block
    }

    fn create_hero_on_power(&mut self, seedPoints: u16, diceVal: u16, seedOffset: usize) -> Hero {
        let offset = seedOffset * 3;
        return Hero::new(seedPoints, 1, diceVal, 0, offset); // no more than 3 in one block
    }

    fn create_trap(&mut self, power: u16, value: u16, diceVal: u16, seedOffset: usize) -> Trap {
        let newID = self.coins.next_id();
        let stat = rand_range(0, 3, seedOffset);
        let offset = seedOffset * 3;
        return Trap::new(power, value, diceVal, stat, newID, offset);  // no more than 3 in one block
    }

    pub fn do_dungeon(&mut self, defender: String, hero_0: u64, hero_1: u64, hero_2: u64) -> Option<BattleLog> {
        let attacker = env::signer_account_id();

        if !self.account_inited(attacker.to_owned()) || !self.account_inited(defender.to_owned()) {
            return None;
        }

        let tier_atk: Option<String> = self.fighters.get(&attacker);
        let tier_def: Option<String> = self.fighters.get(&defender);

        if tier_atk == None || tier_def == None || tier_atk.unwrap() != tier_def.unwrap() {
            return None;
        }

        if self.get_dungeon((*defender).to_string()) == None {
            return None;
        }

        let attacker_heroes = self.usersHeroes.get(&attacker);
        if attacker_heroes == None {
            return None;
        } 

        let attacker_heroes_unwrapped = self.usersHeroes.get(&attacker).unwrap();
        if !attacker_heroes_unwrapped.contains(&hero_0) || !attacker_heroes_unwrapped.contains(&hero_1) || !attacker_heroes_unwrapped.contains(&hero_2) {
            return None;
        }
        let party = vec![hero_0, hero_1, hero_2];
        let mut v = vec![hero_0, hero_1, hero_2];
        v.sort_unstable();
        v.dedup();
        if party.len() != v.len() {
            return None;
        }

        // all checks passed, time to log the battle
        if self.attackLog.get(&attacker) == None {
            self.attackLog.get(&attacker).insert(vec![AttackLogRecord {ts: self.get_time(), account: attacker.to_owned()}]);
        } else {
            self.attackLog.get(&attacker).unwrap().push(AttackLogRecord {ts: self.get_time(), account: attacker.to_owned()});
        }

        let mut res = BattleLog {
            heroes: v.into_iter().map(|index| self.hero_by_id(index).unwrap().to_owned()).collect(),
            traps: self.get_dungeon(defender.to_owned()).unwrap().into_iter().map(|index| self.trap_by_id(index).unwrap().to_owned()).collect(),
            firstTrap: Vec::new(),
            secondTrap: Vec::new(),
            thirdTrap: Vec::new(),
            rewardGold: 0.0,
        };

        let mut seed = 1;
        let mut nTrap = 1;
        let mut deadHeroes = 0;
        let mut deadTraps = 0;
        let mut failed = false;
        
        for trap in self.get_dungeon(defender.to_owned()).unwrap().into_iter() {
            let mut currTrap = self.trap_by_id(trap).unwrap();
            let mut hitHeroes = 0;
            for hero in &party {
                let mut currHero = self.hero_by_id(*hero).unwrap();
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
        
        let basicReward = self.coins.get_user_dungeon_gold(attacker.to_owned()).unwrap();
        if deadHeroes < 3 && !failed {
            res.rewardGold = basicReward * 0.1 * (1.0 + (3 - deadHeroes) as f64 * 0.01 + deadTraps as f64 * 0.01);
            self.coins.transfer_dungeon_gold_from_to(defender.to_owned(), attacker.to_owned(), res.rewardGold);
        }
        return Some(res)
    }

    pub fn test1hero(&self) -> Hero {
        return Hero::new(123, 12, 6, 1, 0)
    }

    pub fn test1trap(&self) -> Trap {
        return Trap::new(10, 7, 6, 0, 1, 0)
    }

    pub fn check_battle_prerequisites(&self, defender: String, hero_0: u64, hero_1: u64, hero_2: u64) -> String {
        let attacker = env::signer_account_id();

        if !self.account_inited(attacker.to_owned()) || !self.account_inited(defender.to_owned()) {
            return "one of accounts not inited".to_string();
        }

        let tier_atk: Option<String> = self.fighters.get(&attacker);
        let tier_def: Option<String> = self.fighters.get(&defender);

        if tier_atk == None || tier_def == None || tier_atk.unwrap() != tier_def.unwrap() {
            return "tiers mismatch".to_string();
        }

        if self.get_dungeon((*defender).to_string()) == None {
            return "no dungeon".to_string();
        }

        let attacker_heroes = self.usersHeroes.get(&attacker);
        if attacker_heroes == None {
            return "no heroes".to_string();
        } 

        let attacker_heroes_unwrapped = self.usersHeroes.get(&attacker).unwrap();
        if !attacker_heroes_unwrapped.contains(&hero_0) || !attacker_heroes_unwrapped.contains(&hero_1) || !attacker_heroes_unwrapped.contains(&hero_2) {
            return "not own heroes".to_string();
        }
        let party = vec![hero_0, hero_1, hero_2];
        let mut v = vec![hero_0, hero_1, hero_2];
        v.sort_unstable();
        v.dedup();
        if party.len() != v.len() {
            return "duplicate heroes".to_string();
        }
        return "should be ok".to_string();
    }
}

