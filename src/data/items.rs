use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use ggez::Context;
use anyhow::{Result, Context as _};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub stackable: bool,
    pub tradeable: bool,
    pub value: u32,
    pub equipment_slot: Option<EquipmentSlot>,
    pub weapon_stats: Option<WeaponStats>,
    pub armor_stats: Option<ArmorStats>,
    pub consumable_effects: Vec<ConsumableEffect>,
    // Add other item properties as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EquipmentSlot {
    Head,
    Body,
    Legs,
    Weapon,
    Shield,
    Amulet,
    Ring,
    Cape,
    // Add other slots as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponStats {
    pub attack_speed: u8,
    pub attack_bonus: i16,
    pub strength_bonus: i16,
    pub magic_bonus: i16,
    pub ranged_bonus: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorStats {
    pub defense_bonus: i16,
    pub magic_defense_bonus: i16,
    pub ranged_defense_bonus: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Heal(u16),
    BoostSkill { skill: SkillType, amount: i16, duration: u32 },
    Poison { damage: u16, duration: u32 },
    // Add other effects as needed
}

pub fn load_item_definitions() -> Vec<ItemDefinition> {
    // Load from JSON or other data source
    vec![]
}

pub fn load_item_definitions(_ctx: &Context) -> Result<Vec<ItemDefinition>> {
    // Load from JSON or other data source
    Ok(vec![])
} 