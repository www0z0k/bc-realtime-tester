#![allow(non_snake_case)]
use crate::*;


#[derive(Default, BorshDeserialize, BorshSerialize, Serialize, Clone)]
pub struct League {
    pub id: u8,
    pub duration: u64,
    pub stash: f64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Coins  {
    pub usersGold: UnorderedMap<AccountId, f64>,
    pub usersDungeonGold: UnorderedMap<AccountId, f64>,
    pub globals: UnorderedMap<String, f64>,
}

pub const FEE: f64 = 0.001;

pub const LEAGUES: [League; 3] = [
    League { id: 0, duration: 30 * 60000 * 1000000, stash: 100.0 },
    League { id: 1, duration: 60 * 60000 * 1000000, stash: 1000.0 },
    League { id: 2, duration: 90 * 60000 * 1000000, stash: 10000.0 }
];

impl Coins {
    pub fn new() -> Coins {
        let coins = Coins {
            globals: UnorderedMap::new(StorageKeys::Globals),
            usersGold: UnorderedMap::new(StorageKeys::UsersGold),
            usersDungeonGold: UnorderedMap::new(StorageKeys::UsersDungeonGold),
        };
        coins
    }

    fn safeUnwrap(opt: Option<f64>) -> f64 {
        if opt == std::option::Option::None {
            0.0
        } else {
            opt.unwrap()
        }
    }

    pub fn transfer_gold_from_system(&mut self, target: String, amount: f64) -> f64 {
        let mut initGold = self.globals.get(&"systemInitialGold".to_owned()).unwrap();
        let mut systemGold = self.globals.get(&"systemGold".to_owned()).unwrap();

        let mut tGold = if self.usersGold.get(&target).to_owned() == std::option::Option::None {
            0.0
        } else {
            self.usersGold.get(&target).unwrap()
        };

        if initGold < amount && systemGold < amount {
            return 0.0
        }
        if systemGold >= amount {
            systemGold -= amount;
            tGold += amount;
            self.usersGold.insert(&target, &tGold);
            self.globals.insert(&"systemGold".to_owned(), &systemGold);
            return amount
        }
        if initGold >= amount {
            initGold -= amount;
            tGold += amount;
            self.usersGold.insert(&target, &tGold);
            self.globals.insert(&"systemInitialGold".to_owned(), &initGold);
        }
        return amount
    }

    pub fn transfer_gold_to_system(&mut self, target: String, amount: f64) -> f64 {
        let mut systemGold = self.globals.get(&"systemGold".to_owned()).unwrap();
        let mut tGold = self.usersGold.get(&target).unwrap();
        if tGold < amount {
            return 0.0
        }
        if systemGold >= amount {
            systemGold += amount;
            tGold -= amount;
            self.usersGold.insert(&target, &tGold);
            self.globals.insert(&"systemGold".to_owned(), &systemGold);
        }
        return amount
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

    pub fn transfer_dungeon_gold_from_to(&mut self, sender: String, target: String, amount: f64) -> f64 {
        let mut sGold = Coins::safeUnwrap(self.usersDungeonGold.get(&sender));
        let mut tGold = Coins::safeUnwrap(self.usersDungeonGold.get(&target));
        if sGold < amount || sender == target {
            return 0.0
        }
        sGold -= amount;
        tGold += amount;
        self.usersDungeonGold.insert(&sender, &sGold);
        self.usersDungeonGold.insert(&target, &tGold);
        return amount
    }

    pub fn fill_dungeon_stash(&mut self, amount: f64) -> f64 {
        let sender = env::signer_account_id();
        let mut dGold = Coins::safeUnwrap(self.usersDungeonGold.get(&sender));
        let mut uGold = self.usersGold.get(&sender).unwrap();
        if uGold < amount {
            return 0.0;
        }
        dGold += amount;
        uGold -= amount;
        self.usersDungeonGold.insert(&sender, &dGold);
        self.usersGold.insert(&sender, &uGold);
        return amount;
    }

    pub fn empty_dungeon_stash(&mut self) -> f64 {
        let sender = env::signer_account_id();
        let mut dGold = Coins::safeUnwrap(self.usersDungeonGold.get(&sender));
        let mut uGold = self.usersGold.get(&sender).unwrap();

        uGold += dGold;
        self.usersDungeonGold.insert(&sender, &0.0);
        self.usersGold.insert(&sender, &uGold);
        return dGold;
    }

    pub fn transfer_gold_to(&mut self, target: String, amount: u64) -> f64 {
        let sender = env::signer_account_id();
        return self.transfer_gold_from_to(sender, target, amount as f64)
    }

    pub fn get_user_dungeon_gold(&self, account_id: String) -> Option<f64> {
        return self.usersDungeonGold.get(&account_id);
    }

    pub fn get_user_gold(&self, account_id: String) -> Option<f64> {
        return self.usersGold.get(&account_id);
    }
}