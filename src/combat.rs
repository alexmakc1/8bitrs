use rand::Rng;
use crate::skills::Skills;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combat {
    pub health: i32,
    pub max_health: i32,
}

impl Combat {
    pub fn new(max_health: i32) -> Self {
        Combat {
            health: max_health,
            max_health,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.health = (self.health - damage).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn attack(&self, attacker_skills: &Skills, defender_skills: &Skills, attack_bonus: i32, strength_bonus: i32, defender_defense_bonus: i32) -> Option<u8> {
        let mut rng = rand::thread_rng();
        
        // Calculate hit chance based on attack level + equipment bonus vs defense level + defense bonus
        let accuracy = 0.5 + ((attacker_skills.attack.get_level() as i32 + attack_bonus) as f32 * 0.01);
        let defense = ((defender_skills.defense.get_level() as i32 + defender_defense_bonus) as f32 * 0.01);
        let hit_chance = (accuracy - defense).max(0.1); // Minimum 10% chance to hit

        if rng.gen::<f32>() <= hit_chance {
            // Calculate damage based on strength level + equipment bonus
            let effective_strength = attacker_skills.strength.get_level() as i32 + strength_bonus;
            let max_hit = 1 + (effective_strength / 10);
            let damage = rng.gen_range(1..=max_hit);
            Some(damage as u8)
        } else {
            None // Miss
        }
    }
} 