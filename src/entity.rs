use ggez::{graphics::{self, Canvas, Color}, GameResult};
use ggez::glam::Vec2;
use rand::Rng;

use crate::combat::Combat;
use crate::skills::Skills;
use crate::sprites::SpriteManager;
use crate::inventory::Item;

#[derive(Debug)]
struct DropTableEntry {
    item: fn() -> Item,
    chance: f32, // Chance out of 100
}

#[derive(Debug)]
struct DropTable {
    entries: Vec<DropTableEntry>,
}

impl DropTable {
    fn roll_drops(&self) -> Vec<Item> {
        let mut rng = rand::thread_rng();
        let mut drops = Vec::new();
        
        for entry in &self.entries {
            if rng.gen_range(0.0..100.0) < entry.chance {
                drops.push((entry.item)());
            }
        }
        
        drops
    }
}

#[derive(Debug)]
pub enum EntityType {
    Goblin(Combat),
    Cow(Combat),
}

impl EntityType {
    fn get_drop_table(&self) -> DropTable {
        match self {
            EntityType::Goblin(_) => DropTable {
                entries: vec![
                    DropTableEntry { item: Item::bronze_sword, chance: 5.0 },     // 5% chance
                    DropTableEntry { item: Item::bronze_helmet, chance: 5.0 },    // 5% chance
                    DropTableEntry { item: Item::bronze_platebody, chance: 5.0 }, // 5% chance
                    DropTableEntry { item: Item::bronze_platelegs, chance: 5.0 }, // 5% chance
                    DropTableEntry { item: Item::bronze_axe, chance: 10.0 },      // 10% chance
                    DropTableEntry { item: Item::fishing_rod, chance: 10.0 },     // 10% chance
                    DropTableEntry { item: Item::bait, chance: 25.0 },           // 25% chance
                    DropTableEntry { item: Item::tinderbox, chance: 10.0 },      // 10% chance
                ],
            },
            EntityType::Cow(_) => DropTable {
                entries: vec![
                    DropTableEntry { item: Item::beef, chance: 100.0 },      // 100% chance
                    DropTableEntry { item: Item::cow_hide, chance: 100.0 },  // 100% chance
                    DropTableEntry { item: Item::bones, chance: 100.0 },     // 100% chance
                ],
            },
        }
    }
}

pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub entity_type: EntityType,
    pub respawn_timer: Option<f32>,
}

impl Entity {
    pub fn new_goblin(x: f32, y: f32) -> Self {
        Entity {
            x,
            y,
            entity_type: EntityType::Goblin(Combat::new(10)), // Goblins have 10 HP
            respawn_timer: None,
        }
    }

    pub fn new_cow(x: f32, y: f32) -> Self {
        Entity {
            x,
            y,
            entity_type: EntityType::Cow(Combat::new(8)), // Cows have 8 HP
            respawn_timer: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(timer) = &mut self.respawn_timer {
            *timer -= dt;
            if *timer <= 0.0 {
                self.respawn_timer = None;
                // Reset goblin health
                let combat = match &mut self.entity_type {
                    EntityType::Goblin(combat) => combat,
                    EntityType::Cow(combat) => combat,
                };
                *combat = Combat::new(10);
            }
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, sprites: &SpriteManager) -> GameResult {
        self.draw_with_offset(canvas, 0.0, 0.0, sprites)
    }

    pub fn draw_with_offset(&self, canvas: &mut Canvas, offset_x: f32, offset_y: f32, sprites: &SpriteManager) -> GameResult {
        match &self.entity_type {
            EntityType::Goblin(combat) => {
                if !combat.is_dead() {
                    if let Some(sprite) = sprites.get_sprite("goblin") {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(self.x - offset_x - 16.0, self.y - offset_y - 16.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );
                    }
                }
            }
            EntityType::Cow(combat) => {
                if !combat.is_dead() {
                    if let Some(sprite) = sprites.get_sprite("cow") {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(self.x - offset_x - 16.0, self.y - offset_y - 16.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );
                    }
                }
            }
        }

        // Draw health bar if entity is alive
        if let Some(combat) = self.get_combat() {
            if !combat.is_dead() {
                let health_percent = combat.health as f32 / combat.max_health as f32;
                
                // Black background
                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(self.x - offset_x - 16.0, self.y - offset_y - 26.0))
                        .scale(Vec2::new(32.0, 5.0))
                        .color(Color::BLACK)
                );

                // Green health bar
                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(self.x - offset_x - 16.0, self.y - offset_y - 26.0))
                        .scale(Vec2::new(32.0 * health_percent, 5.0))
                        .color(Color::GREEN)
                );
            }
        }

        Ok(())
    }

    pub fn get_drops(&self) -> Vec<Item> {
        self.entity_type.get_drop_table().roll_drops()
    }

    pub fn interact(&mut self, _player_skills: &mut Skills) -> Option<Vec<Item>> {
        if self.respawn_timer.is_some() {
            return None;
        }

        match &mut self.entity_type {
            EntityType::Goblin(combat) => {
                if combat.is_dead() {
                    self.respawn_timer = Some(5.0); // 5 seconds to respawn
                    Some(self.get_drops())
                } else {
                    None
                }
            }
            EntityType::Cow(combat) => {
                if combat.is_dead() {
                    self.respawn_timer = Some(5.0); // 5 seconds to respawn
                    Some(self.get_drops())
                } else {
                    None
                }
            }
        }
    }

    pub fn get_combat(&self) -> Option<&Combat> {
        match &self.entity_type {
            EntityType::Goblin(combat) | EntityType::Cow(combat) => Some(combat),
        }
    }

    pub fn get_combat_mut(&mut self) -> Option<&mut Combat> {
        match &mut self.entity_type {
            EntityType::Goblin(combat) | EntityType::Cow(combat) => Some(combat),
        }
    }

    pub fn is_near(&self, x: f32, y: f32) -> bool {
        let dx = self.x - x;
        let dy = self.y - y;
        let distance = (dx * dx + dy * dy).sqrt();
        distance < 40.0 // Interaction range of 40 pixels
    }

    pub fn is_alive(&self) -> bool {
        match &self.entity_type {
            EntityType::Goblin(combat) | EntityType::Cow(combat) => !combat.is_dead() && self.respawn_timer.is_none(),
        }
    }

    pub fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
} 