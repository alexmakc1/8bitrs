use ggez::{Context, GameResult};
use ggez::graphics::{self, Image};
use std::collections::HashMap;

pub struct SpriteManager {
    sprites: HashMap<String, Image>,
}

impl SpriteManager {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut sprites = HashMap::new();

        // Load player sprite
        sprites.insert(
            "player".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/player.png")
            )?
        );

        // Load environment sprites
        sprites.insert(
            "tree".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/tree.png")
            )?
        );

        sprites.insert(
            "tree_stump".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/tree_stump.png")
            )?
        );

        sprites.insert(
            "wall".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/wall.png")
            )?
        );

        sprites.insert(
            "water".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/water.png")
            )?
        );

        sprites.insert(
            "road".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/road.png")
            )?
        );

        sprites.insert(
            "fence".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/fence.png")
            )?
        );

        sprites.insert(
            "castle_wall".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/castle_wall.png")
            )?
        );

        sprites.insert(
            "castle_door".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/castle_door.png")
            )?
        );

        sprites.insert(
            "castle_stairs".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/castle_stairs.png")
            )?
        );

        sprites.insert(
            "bridge".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/bridge.png")
            )?
        );

        sprites.insert(
            "path".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/path.png")
            )?
        );

        sprites.insert(
            "goblin".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/goblin.png")
            )?
        );

        sprites.insert(
            "fire".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/fire.png")
            )?
        );

        sprites.insert(
            "fishing_spot".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/fishing_spot.png")
            )?
        );

        // Load item sprites
        sprites.insert(
            "sword".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/sword.png")
            )?
        );

        sprites.insert(
            "axe".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/axe.png")
            )?
        );

        sprites.insert(
            "logs".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/logs.png")
            )?
        );

        sprites.insert(
            "fish".to_string(),
            Image::from_bytes(
                ctx,
                include_bytes!("../assets/sprites/fish.png")
            )?
        );

        Ok(SpriteManager { sprites })
    }

    pub fn get_sprite(&self, name: &str) -> Option<&Image> {
        self.sprites.get(name)
    }
} 