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
                    DropTableEntry { item: Item::bones, chance: 100.0 },         // 100% chance
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
                    DropTableEntry { item: Item::raw_beef, chance: 100.0 },   // 100% chance
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
    spawn_x: f32,
    spawn_y: f32,
    movement_timer: f32,
    movement_target: Option<(f32, f32)>,
}

impl Entity {
    pub fn new_goblin(x: f32, y: f32) -> Self {
        Entity {
            x,
            y,
            entity_type: EntityType::Goblin(Combat::new(10)), // Goblins have 10 HP
            respawn_timer: None,
            spawn_x: x,
            spawn_y: y,
            movement_timer: 0.0,
            movement_target: None,
        }
    }

    pub fn new_cow(x: f32, y: f32) -> Self {
        Entity {
            x,
            y,
            entity_type: EntityType::Cow(Combat::new(8)), // Cows have 8 HP
            respawn_timer: None,
            spawn_x: x,
            spawn_y: y,
            movement_timer: 0.0,
            movement_target: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(timer) = &mut self.respawn_timer {
            *timer -= dt;
            if *timer <= 0.0 {
                self.respawn_timer = None;
                // Reset health and position
                let combat = match &mut self.entity_type {
                    EntityType::Goblin(combat) => combat,
                    EntityType::Cow(combat) => combat,
                };
                *combat = Combat::new(10);
                self.x = self.spawn_x;
                self.y = self.spawn_y;
                self.movement_target = None;
            }
            return;
        }

        // Update movement
        self.movement_timer -= dt;
        if self.movement_timer <= 0.0 {
            let mut rng = rand::thread_rng();
            // 30% chance to start moving
            if rng.gen_bool(0.3) {
                // Pick a random point within 100 pixels of spawn point
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let distance = rng.gen_range(0.0..100.0);
                let target_x = self.spawn_x + angle.cos() * distance;
                let target_y = self.spawn_y + angle.sin() * distance;
                self.movement_target = Some((target_x, target_y));
            } else {
                self.movement_target = None;
            }
            self.movement_timer = rng.gen_range(2.0..5.0); // Set next movement check in 2-5 seconds
        }

        // Move towards target if we have one
        if let Some((target_x, target_y)) = self.movement_target {
            let dx = target_x - self.x;
            let dy = target_y - self.y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance > 5.0 { // Only move if we're not very close to target
                let speed = 50.0; // pixels per second
                let move_distance = speed * dt;
                let ratio = move_distance / distance;
                self.x += dx * ratio;
                self.y += dy * ratio;
            } else {
                self.movement_target = None;
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

    pub fn interact(&self, skills: &mut Skills) -> Option<Vec<Item>> {
        match &self.entity_type {
            EntityType::Goblin(_) => {
                // 50% chance to drop bones
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) {
                    Some(vec![Item::bones()])
                } else {
                    None
                }
            }
            EntityType::Cow(_) => {
                Some(vec![Item::raw_beef()])
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