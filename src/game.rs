#![allow(non_snake_case)]
use crate::*;

/*
cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/tribe_terra.wasm res/tribe_terra.wasm && near dev-deploy res/tribe_terra.wasm

near view dev-1649287974163-15947483174306 get_user_heroes '{"account_id": "www0rker.testnet"}'
near view dev-1649287974163-15947483174306 hero_by_id '{"id": 1}'

near call dev-1649287974163-15947483174306 set_interval '{"id": "1"}' --accountId 'www0rker.testnet'
near view dev-1649287974163-15947483174306 get_interval '{"id": "1"}'

near call dev-1649287974163-15947483174306 add_to_stat '{"id": 1, "stat": "vitality"}' --accountId 'www0rker.testnet'
near call dev-1649287974163-15947483174306 init --accountId 'www0rker.testnet'

*/
#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Fillable {
    pub current: u16,
    pub full: u16,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Fillable32 {
    pub current: u32,
    pub full: u32,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Stats {
    pub vitality: u16,
    pub strength: u16,
    pub agility: u16,
    pub intelligence: u16,
}

impl Stats {
    pub fn sum(&self) -> u16 {
        self.vitality + self.strength + self.intelligence + self.vitality
    }
}

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct SingleBattleResult {
    pub heroID: u64,
    pub trapID: u64,
    pub damageToTrap: u16,
    pub damageToHero: u16,
    pub heroHP: u16,
    pub trapHP: u16,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct BattleLog {
    pub firstTrap: Vec<SingleBattleResult>,
    pub secondTrap: Vec<SingleBattleResult>,
    pub thirdTrap: Vec<SingleBattleResult>,
    pub rewardGold: f64,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Trap {
    pub id: u64,
    pub stat: u16,
    pub checkDice: u16,
    pub power: u16,
    pub value: u16,
    pub health: Fillable,
    pub enabled: bool,
}

const LEVELS: [u32; 60] = [500, 1400, 2900, 4900, 7600, 10900, 14900, 19700, 25300, 31700, 39100, 47500, 66500, 88500, 112500, 138500, 167500, 199500, 234500, 272500, 337500, 407500, 482500, 562500, 647500, 737500, 832500, 937500, 1047500, 1167500, 1337500, 1517500, 1707500, 1907500, 2117500, 2347500, 2587500, 2837500, 3097500, 3377500, 3737500, 4117500, 4517500, 4937500, 5377500, 5837500, 6317500, 6817500, 7347500, 7897500, 8587500, 9307500, 10057500, 10837500, 11647500, 12487500, 13367500, 14277500, 15227500, 16207500];

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Hero {
    pub id: u64,
    pub checkDice: u16,
    
    baseStats: Stats,
    improvedStats: Stats,
    
    pub health: Fillable,
    pub mana: Fillable,
    pub experience: Fillable32,
    
    pub name: String,

    pub race: u8, // 0..22
    pub class: u8,
    pub level: u8,
    pub kMana: u8,
    pub kHP: u8,
    pub unusedPoints: u8,
}

impl Trap {
    pub fn new(power: u16, value: u16, diceVal: u16, stat: u16, id: u64, seedOffset: usize) -> Trap {
        let pow: u16 = rand_range(1, value, seedOffset);
        let hp: u16 = (power - pow) * 63;
        let trap = Trap {
            id: id,
            stat: stat,
            checkDice: diceVal,

            power: power,
            value: pow,
            
            enabled: true,
            
            health: Fillable { current: hp, full: hp, },
        };
        trap
    }

    pub(crate) fn takeDamage(&mut self, amount: u16) {
        self.health.current = if self.health.current < amount { 0 } else { self.health.current - amount };
        if amount > 0 {
            self.enabled = false;
        }
    }
}

impl Hero {
    pub fn new(seedPoints: u16, minStat: u16, diceVal: u16, id: u64, seedOffset: usize) -> Hero {
        let mut seed = seedPoints - minStat * 3;
        let vitality = rand_range(1, seed, seedOffset);
        seed = seed + minStat - vitality;
        let strength = rand_range(1, seed, seedOffset + 1);
        seed = seed + minStat - strength;
        let intelligence = rand_range(1, seed, seedOffset + 2);
        seed = seed + minStat - intelligence;
        let agility = seed;

        let class = 
        if vitality >= strength && vitality >= agility && vitality >= intelligence {
            0
        } else if strength >= agility && strength >= intelligence {
            1
        } else if agility >= intelligence {
            2
        } else {
            3
        };

        let race = rand_range(0, 22, seedOffset) as u8;
        let fullHP = vitality * 25 * (4 - class);
        let fullMana = intelligence * 25 * (class + 1);

        let hero = Hero {
            id: id,
            checkDice: diceVal,

            baseStats: Stats {
                vitality: vitality,
                strength: strength,
                agility: agility,
                intelligence: intelligence,
            },
            improvedStats: Stats {
                vitality: 0,
                strength: 0,
                agility: 0,
                intelligence: 0,
            },

            health: Fillable { current: fullHP, full: fullHP, },
            mana: Fillable { current: fullMana, full: fullMana, },
            experience: Fillable32 { current: 0, full: LEVELS[0], },

            name: "Aboba".to_string(),

            race: race,
            level: 1,
            
            class: class as u8,
            kHP: 25 * (4 - class) as u8,
            kMana: 25 * (class + 1) as u8,
            unusedPoints: 0,
        };
        hero
    }

    pub fn basePower(&mut self) -> u16 {
        self.baseStats.sum()
    }

    pub fn totalPower(&mut self) -> u16 {
        self.baseStats.sum() + self.improvedStats.sum()
    }

    pub fn improveStat(&mut self, stat: String) {
        if self.unusedPoints > 0 {
            if (stat == "vitality") {
                self.improvedStats.vitality += 1;
                self.unusedPoints -= 1;
            } else if (stat == "strength") {
                self.improvedStats.strength += 1;
                self.unusedPoints -= 1;
            } else if (stat == "agility") {
                self.improvedStats.agility += 1;
                self.unusedPoints -= 1;
            } else if (stat == "intelligence") {
                self.improvedStats.intelligence += 1;
                self.unusedPoints -= 1;
            }
        }
    }

    /** (taken, given) */ 
    pub fn rollStatAgainst(&mut self, statIndex: u16, value: u16, seedOffset: usize) -> (u16, u16) {
        let mut roll = rand_range(1, self.checkDice, seedOffset);
        if statIndex == 0 {
            roll += self.baseStats.vitality + self.improvedStats.vitality;
        } else if statIndex == 1 {
            roll += self.baseStats.strength + self.improvedStats.strength;
        } else if statIndex == 2 {
            roll += self.baseStats.agility + self.improvedStats.agility;
        } else if statIndex == 3 {
            roll += self.baseStats.intelligence + self.improvedStats.intelligence;
        }
        
        if value > roll {
            let taken = value - roll;
            self.takeDamage(taken);
            return (taken, 0)
        } else {
            // TODO add exp here
            self.experience.current += u32::from((roll - value) * 10);
            self.validateExp();
        }
        return (0, roll - value)
    }

    fn takeDamage(&mut self, amount: u16) {
        self.health.current = if self.health.current < amount { 0 } else { self.health.current - amount }
    }
    fn recieveHealing(&mut self, amount: u16) {
        self.health.current = if self.health.current + amount > self.health.full { 0 } else { self.health.current + amount }
    }

    fn validateExp(&mut self) {
        if self.experience.current >= self.experience.full {
            self.level += 1;
            self.unusedPoints = ((self.basePower() / 10) as f64).ceil() as u8;
            self.experience.full = LEVELS[(self.level - 1)  as usize];
        }
    }
}