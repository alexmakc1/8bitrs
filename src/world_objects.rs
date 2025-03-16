use ggez::{graphics::{self, Canvas}, GameResult};
use ggez::glam::Vec2;
use crate::sprites::SpriteManager;
use crate::skills::Skills;
use crate::inventory::{Item, ItemType, ToolType};
use rand::Rng;

#[derive(Debug, Clone)]
pub enum ObjectType {
    Wall,
    Tree,
    Water,
    Road,
    Fence,
    CastleWall,
    CastleDoor,
    CastleStairs,
    Bridge,
    Path,
    BankChest,
}

#[derive(Debug)]
pub struct WorldObject {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub object_type: ObjectType,
    pub blocks_movement: bool,
    pub health: u8,
    pub fallen: bool,
}

impl WorldObject {
    pub fn new(x: f32, y: f32, object_type: ObjectType) -> Self {
        let (width, height, blocks_movement, health) = match object_type {
            ObjectType::Wall | ObjectType::CastleWall => (40.0, 40.0, true, 255),
            ObjectType::Tree => (32.0, 32.0, true, 3),
            ObjectType::Water => (40.0, 40.0, true, 255),
            ObjectType::Road => (40.0, 40.0, false, 255),
            ObjectType::Fence => (40.0, 8.0, true, 255),
            ObjectType::CastleDoor => (40.0, 40.0, false, 255),
            ObjectType::CastleStairs => (40.0, 40.0, false, 255),
            ObjectType::Bridge => (40.0, 40.0, false, 255),
            ObjectType::Path => (40.0, 40.0, false, 255),
            ObjectType::BankChest => (40.0, 40.0, false, 255),
        };

        Self {
            x,
            y,
            width,
            height,
            object_type,
            blocks_movement,
            health,
            fallen: false,
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, offset_x: f32, offset_y: f32, sprites: &SpriteManager) -> GameResult {
        let sprite_name = match &self.object_type {
            ObjectType::Tree if self.fallen => "tree_stump",
            _ => self.object_type.get_sprite_name(),
        };

        if let Some(sprite) = sprites.get_sprite(sprite_name) {
            canvas.draw(
                sprite,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.x - offset_x - self.width/2.0, self.y - offset_y - self.height/2.0))
                    .scale(Vec2::new(2.0, 2.0))
            );
        }
        Ok(())
    }

    pub fn collides_with(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        if !self.blocks_movement {
            return false;
        }

        // Simple AABB collision detection
        let self_left = self.x - self.width/2.0;
        let self_right = self.x + self.width/2.0;
        let self_top = self.y - self.height/2.0;
        let self_bottom = self.y + self.height/2.0;

        let other_left = x - width/2.0;
        let other_right = x + width/2.0;
        let other_top = y - height/2.0;
        let other_bottom = y + height/2.0;

        self_left < other_right &&
        self_right > other_left &&
        self_top < other_bottom &&
        self_bottom > other_top
    }

    pub fn is_chopped(&self) -> bool {
        matches!(self.object_type, ObjectType::Tree) && self.health == 0
    }

    pub fn try_chop(&mut self, skills: &Skills, axe: Option<&Item>) -> bool {
        if self.is_chopped() || !matches!(self.object_type, ObjectType::Tree) || self.fallen {
            return false;
        }

        if let Some(item) = axe {
            if let ItemType::Tool(ToolType::Axe { woodcutting_level }) = &item.item_type {
                if u32::from(skills.woodcutting.get_level()) >= *woodcutting_level {
                    return true;
                }
            }
        }
        false
    }

    pub fn set_chopped(&mut self) {
        if matches!(self.object_type, ObjectType::Tree) {
            self.health = 0;
            self.fallen = true;
            self.blocks_movement = false;  // Allow walking over stumps
        }
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

impl ObjectType {
    pub fn get_sprite_name(&self) -> &str {
        match self {
            ObjectType::Wall => "wall",
            ObjectType::Tree => "tree",
            ObjectType::Water => "water",
            ObjectType::Road => "road",
            ObjectType::Fence => "fence",
            ObjectType::CastleWall => "castle_wall",
            ObjectType::CastleDoor => "castle_door",
            ObjectType::CastleStairs => "castle_stairs",
            ObjectType::Bridge => "bridge",
            ObjectType::Path => "path",
            ObjectType::BankChest => "bank_chest",
        }
    }
} 