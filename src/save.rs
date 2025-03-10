use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};
use ggez::Context;

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
    fn get_save_path(ctx: &Context) -> PathBuf {
        let mut path = ctx.fs.user_config_dir().to_path_buf();
        path.push("save_game.json");
        path
    }

    pub fn save_to_file(&self, ctx: &Context) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let save_path = Self::get_save_path(ctx);
        
        // Create parent directories if they don't exist
        if let Some(parent) = save_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write the save file
        fs::write(&save_path, json)?;
        println!("Game saved to: {}", save_path.display());
        Ok(())
    }

    pub fn load_from_file(ctx: &Context) -> std::io::Result<Option<SaveData>> {
        let save_path = Self::get_save_path(ctx);
        
        if !save_path.exists() {
            println!("No save file found at: {}", save_path.display());
            return Ok(None);
        }

        match fs::read_to_string(&save_path) {
            Ok(json) => {
                match serde_json::from_str(&json) {
                    Ok(save_data) => {
                        println!("Successfully loaded save from: {}", save_path.display());
                        Ok(Some(save_data))
                    }
                    Err(e) => {
                        println!("Error parsing save file: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                println!("Error reading save file: {}", e);
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