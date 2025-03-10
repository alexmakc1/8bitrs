use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub attack: Skill,
    pub strength: Skill,
    pub defense: Skill,
    pub woodcutting: Skill,
    pub firemaking: Skill,
    pub fishing: Skill,
    pub cooking: Skill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    level: u8,
    experience: u32,
}

impl Skills {
    pub fn new() -> Self {
        Skills {
            attack: Skill::new(),
            strength: Skill::new(),
            defense: Skill::new(),
            woodcutting: Skill::new(),
            firemaking: Skill::new(),
            fishing: Skill::new(),
            cooking: Skill::new(),
        }
    }

    pub fn gain_attack_xp(&mut self, amount: u32) {
        self.attack.add_experience(amount);
        println!("Gained {} Attack XP. New level: {}", amount, self.attack.get_level());
    }

    pub fn gain_strength_xp(&mut self, amount: u32) {
        self.strength.add_experience(amount);
        println!("Gained {} Strength XP. New level: {}", amount, self.strength.get_level());
    }

    pub fn gain_defense_xp(&mut self, amount: u32) {
        self.defense.add_experience(amount);
        println!("Gained {} Defense XP. New level: {}", amount, self.defense.get_level());
    }

    pub fn gain_woodcutting_xp(&mut self, amount: u32) {
        self.woodcutting.add_experience(amount);
        println!("Gained {} Woodcutting XP. New level: {}", amount, self.woodcutting.get_level());
    }

    pub fn gain_firemaking_xp(&mut self, amount: u32) {
        self.firemaking.add_experience(amount);
        println!("Gained {} Firemaking XP. New level: {}", amount, self.firemaking.get_level());
    }

    pub fn gain_fishing_xp(&mut self, xp: u32) {
        self.fishing.add_experience(xp);
    }

    pub fn gain_cooking_xp(&mut self, amount: u32) {
        self.cooking.add_experience(amount);
        println!("Gained {} Cooking XP. New level: {}", amount, self.cooking.get_level());
    }
}

impl Skill {
    pub fn new() -> Self {
        Skill {
            level: 1,
            experience: 0,
        }
    }

    pub fn get_level(&self) -> u8 {
        self.level
    }

    pub fn get_experience(&self) -> u32 {
        self.experience
    }

    pub fn add_experience(&mut self, exp: u32) {
        self.experience += exp;
        self.update_level();
    }

    fn update_level(&mut self) {
        // RuneScape's experience formula
        let mut level = 1;
        let mut points = 0;
        
        while level < 99 {
            points += ((level as f64 + 300.0 * 2.0_f64.powf(level as f64 / 7.0)) / 4.0) as u32;
            if points > self.experience {
                break;
            }
            level += 1;
        }
        
        self.level = level;
    }
} 