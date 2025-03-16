use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterDefinition {
    pub id: u32,
    pub name: String,
    pub level: u16,
    pub hitpoints: u16,
    pub attack_speed: u8,
    pub attack_bonus: i16,
    pub strength_bonus: i16,
    pub defense_bonus: i16,
    pub magic_bonus: i16,
    pub magic_defense_bonus: i16,
    pub ranged_bonus: i16,
    pub ranged_defense_bonus: i16,
    pub combat_style: CombatStyle,
    pub drop_table: Vec<Drop>,
    pub skills: Vec<MonsterSkill>,
    // Add other monster properties as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatStyle {
    Melee,
    Ranged,
    Magic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drop {
    pub item_id: u32,
    pub quantity: (u16, u16), // Min and max quantity
    pub chance: f32, // Drop chance (0.0 to 1.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterSkill {
    pub skill_type: SkillType,
    pub level: u16,
}

pub fn load_monster_definitions() -> Vec<MonsterDefinition> {
    // Load from JSON or other data source
    vec![]
} 