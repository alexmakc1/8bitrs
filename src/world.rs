use ggez::{graphics::{self, Canvas, Color}, GameResult};
use ggez::glam::Vec2;
use rand::Rng;
use std::time::Duration;

use crate::skills::Skills;
use crate::inventory::{Item, ItemType, ToolType, ResourceType};
use crate::sprites::SpriteManager;

#[derive(Debug)]
pub struct Tree {
    pub x: f32,
    pub y: f32,
    pub health: u8,
    pub respawn_timer: Option<f32>,
    tree_type: TreeType,
    pub fallen: bool,
}

#[derive(Debug, PartialEq)]
enum TreeType {
    Normal,
    Wall,
}

impl Tree {
    pub fn new(x: f32, y: f32) -> Self {
        Tree {
            x,
            y,
            health: 3,
            respawn_timer: None,
            tree_type: TreeType::Normal,
            fallen: false,
        }
    }

    pub fn new_wall(x: f32, y: f32) -> Self {
        Tree {
            x,
            y,
            health: 255, // Walls can't be chopped
            respawn_timer: None,
            tree_type: TreeType::Wall,
            fallen: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(timer) = &mut self.respawn_timer {
            *timer -= dt;
            if *timer <= 0.0 {
                self.health = 3;
                self.respawn_timer = None;
                self.fallen = false;
            }
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, sprites: &SpriteManager) -> GameResult {
        self.draw_with_offset(canvas, 0.0, 0.0, sprites)
    }

    pub fn draw_with_offset(&self, canvas: &mut Canvas, offset_x: f32, offset_y: f32, sprites: &SpriteManager) -> GameResult {
        let sprite_name = if self.tree_type == TreeType::Wall {
            "wall"
        } else if self.fallen {
            "tree_stump"
        } else {
            "tree"
        };

        if let Some(sprite) = sprites.get_sprite(sprite_name) {
            canvas.draw(
                sprite,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.x - offset_x - 16.0, self.y - offset_y - 16.0))
                    .scale(Vec2::new(2.0, 2.0))
            );
        }
        Ok(())
    }

    pub fn is_near(&self, x: f32, y: f32) -> bool {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt() < 40.0
    }

    pub fn is_chopped(&self) -> bool {
        match self.tree_type {
            TreeType::Normal => self.health == 0,
            TreeType::Wall => false, // Walls can't be chopped
        }
    }

    pub fn try_chop(&mut self, skills: &Skills, axe: Option<&Item>) -> bool {
        if self.is_chopped() || matches!(self.tree_type, TreeType::Wall) {
            return false;
        }

        if let Some(item) = axe {
            if let ItemType::Tool(ToolType::Axe { woodcutting_level }) = &item.item_type {
                if u32::from(skills.woodcutting.get_level()) >= *woodcutting_level {
                    self.health -= 1;
                    if self.is_chopped() {
                        self.fallen = true;
                        self.respawn_timer = Some(30.0); // Tree respawns after 30 seconds
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn get_random_logs(&self) -> u32 {
        if self.is_chopped() {
            let mut rng = rand::thread_rng();
            rng.gen_range(1..=35)
        } else {
            0
        }
    }
}

#[derive(Debug)]
pub struct Fire {
    pub x: f32,
    pub y: f32,
    pub lifetime: f32,
}

impl Fire {
    pub fn new(x: f32, y: f32) -> Self {
        Fire {
            x,
            y,
            lifetime: 60.0, // Fire lasts for 60 seconds
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.lifetime -= dt;
    }

    pub fn is_expired(&self) -> bool {
        self.lifetime <= 0.0
    }

    pub fn draw(&self, canvas: &mut Canvas, sprites: &SpriteManager) -> GameResult {
        self.draw_with_offset(canvas, 0.0, 0.0, sprites)
    }

    pub fn draw_with_offset(&self, canvas: &mut Canvas, offset_x: f32, offset_y: f32, sprites: &SpriteManager) -> GameResult {
        if let Some(sprite) = sprites.get_sprite("fire") {
            canvas.draw(
                sprite,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.x - offset_x - 16.0, self.y - offset_y - 16.0))
                    .scale(Vec2::new(2.0, 2.0))
            );
        }
        Ok(())
    }

    pub fn is_near(&self, x: f32, y: f32) -> bool {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt() < 40.0
    }

    pub fn try_cook(&self, raw_item: &Item, cooking_level: u8) -> Option<Item> {
        let mut rng = rand::thread_rng();
        
        match &raw_item.item_type {
            ItemType::Resource(ResourceType::RawFish { cooking_level: req_level, burn_level }) => {
                if u32::from(cooking_level) >= *req_level {
                    // Higher cooking level = less chance to burn
                    let burn_chance = if u32::from(cooking_level) >= *burn_level {
                        0.0 // Never burn after reaching burn level
                    } else {
                        0.6 - (cooking_level as f64 * 0.02) // 2% less chance to burn per level
                    };
                    
                    if rng.gen_bool(burn_chance) {
                        Some(Item::burnt_fish())
                    } else {
                        Some(Item::cooked_fish())
                    }
                } else {
                    None
                }
            }
            ItemType::Resource(ResourceType::RawBeef { cooking_level: req_level, burn_level }) => {
                if u32::from(cooking_level) >= *req_level {
                    // Higher cooking level = less chance to burn
                    let burn_chance = if u32::from(cooking_level) >= *burn_level {
                        0.0 // Never burn after reaching burn level
                    } else {
                        0.4 - (cooking_level as f64 * 0.02) // 2% less chance to burn per level, starts at 40% instead of 60%
                    };
                    
                    if rng.gen_bool(burn_chance) {
                        Some(Item::burnt_beef())
                    } else {
                        Some(Item::cooked_beef())
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct FishingSpot {
    pub x: f32,
    pub y: f32,
    lifetime: f32,
    pub fish_type: FishType,
}

#[derive(Clone)]
pub enum FishType {
    Shrimp,
    Trout,
}

impl FishingSpot {
    pub fn new(x: f32, y: f32, fish_type: FishType) -> Self {
        FishingSpot {
            x,
            y,
            lifetime: 30.0, // Spots last 30 seconds
            fish_type,
        }
    }

    pub fn is_near(&self, x: f32, y: f32) -> bool {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt() < 40.0
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        self.lifetime > 0.0 // Return true if spot is still active
    }

    pub fn draw(&self, canvas: &mut Canvas, sprites: &SpriteManager) -> GameResult {
        self.draw_with_offset(canvas, 0.0, 0.0, sprites)
    }

    pub fn draw_with_offset(&self, canvas: &mut Canvas, offset_x: f32, offset_y: f32, sprites: &SpriteManager) -> GameResult {
        if let Some(sprite) = sprites.get_sprite("fishing_spot") {
            canvas.draw(
                sprite,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.x - offset_x - 16.0, self.y - offset_y - 16.0))
                    .scale(Vec2::new(2.0, 2.0))
            );
        }
        Ok(())
    }

    pub fn try_fish(&self, skills: &Skills, rod: Option<&Item>, bait: bool) -> Option<Item> {
        match &self.fish_type {
            FishType::Shrimp => {
                if skills.fishing.get_level() >= 1 && rod.is_some() {
                    if rand::thread_rng().gen_bool(0.4) { // 40% success rate
                        Some(Item::raw_shrimp())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            FishType::Trout => {
                if skills.fishing.get_level() >= 15 && rod.is_some() && bait {
                    if rand::thread_rng().gen_bool(0.3) { // 30% success rate
                        Some(Item::raw_trout())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
} 