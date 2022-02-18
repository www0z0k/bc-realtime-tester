#![allow(non_snake_case)]
use crate::*;

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Fillable {
    pub current: u16,
    pub full: u16,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Stats {
    pub vitality: u16,
    pub strength: u16,
    pub agility: u16,
    pub intelligence: u16,
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

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Hero {
    pub id: u64,
    pub checkDice: u16,
    
    baseStats: Stats,
    improvedStats: Stats,
    
    pub health: Fillable,
    pub mana: Fillable,
    pub experience: Fillable,
    
    pub name: String,

    pub race: u8, // 0..22
    pub class: u8,
    pub level: u8,
    pub kMana: u8,
    pub kHP: u8,
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
        let agility = rand_range(1, seed, seedOffset + 2);
        seed = seed + minStat - agility;
        let intelligence = seed;

        let class = if vitality > strength && vitality > agility && vitality > intelligence {
            0
        } else if strength > agility && strength > intelligence {
            1
        } else if agility > intelligence {
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
            experience: Fillable { current: 0, full: 50, },

            name: "Aboba".to_string(),

            race: race,
            level: 1,
            
            class: class as u8,
            kHP: 25 * (4 - class) as u8,
            kMana: 25 * (class + 1) as u8,
        };
        hero
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
            self.experience.current += (roll - value) * 10;
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

    }
}