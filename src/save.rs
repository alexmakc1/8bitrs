use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

use crate::skills::Skills;
use crate::inventory::Inventory;
use crate::equipment::Equipment;
use crate::combat::Combat;

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    // Player position
    pub player_x: f32,
    pub player_y: f32,
    
    // Skills and experience
    pub skills: Skills,
    
    // Combat stats
    pub health: i32,
    pub max_health: i32,
    
    // Items
    pub inventory: Inventory,
    pub equipment: Equipment,
}

impl SaveData {
    pub fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(filename, json)?;
        println!("Game saved successfully!");
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> std::io::Result<Option<SaveData>> {
        if !Path::new(filename).exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(filename)?;
        match serde_json::from_str(&json) {
            Ok(save_data) => {
                println!("Game loaded successfully!");
                Ok(Some(save_data))
            },
            Err(e) => {
                println!("Error loading save file: {}", e);
                Ok(None)
            }
        }
    }
}

// Create save data from current game state
pub fn create_save_data(
    player_x: f32,
    player_y: f32,
    skills: &Skills,
    player_combat: &Combat,
    inventory: &Inventory,
    equipment: &Equipment,
) -> SaveData {
    SaveData {
        player_x,
        player_y,
        skills: skills.clone(),
        health: player_combat.health,
        max_health: player_combat.max_health,
        inventory: inventory.clone(),
        equipment: equipment.clone(),
    }
} 