use crate::combat::Combat;
use serde::{Serialize, Deserialize};
use ggez::graphics;
use ggez::graphics::Canvas;
use ggez::GameResult;
use ggez::glam::Vec2;
use crate::SpriteManager;
use ggez::graphics::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Weapon(WeaponStats),
    Armor(ArmorStats),
    Food(i32), // healing amount
    Tool(ToolType),
    Resource(ResourceType),
    Currency(u32), // value in GP
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponStats {
    pub attack_bonus: i32,
    pub strength_bonus: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorStats {
    pub defense_bonus: i32,
    pub slot: ArmorSlot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArmorSlot {
    Head,
    Body,
    Legs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Axe { woodcutting_level: u32 },
    Tinderbox,
    FishingRod { fishing_level: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Logs { firemaking_level: u32 },
    RawFish { cooking_level: u32, burn_level: u32 },
    CookedFish { healing: u32 },
    BurntFish,
    RawBeef { cooking_level: u32, burn_level: u32 },
    BurntBeef,
    Bait,
    Hide,
    Bones,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub stackable: bool,
    pub quantity: u32,
}

impl Item {
    pub fn gp(amount: u32) -> Self {
        Item {
            name: "GP".to_string(),
            item_type: ItemType::Currency(1),
            stackable: true,
            quantity: amount,
        }
    }

    pub fn bronze_sword() -> Self {
        Item {
            name: "Bronze Sword".to_string(),
            item_type: ItemType::Weapon(WeaponStats {
                attack_bonus: 4,
                strength_bonus: 3,
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn bronze_helmet() -> Self {
        Item {
            name: "Bronze Helmet".to_string(),
            item_type: ItemType::Armor(ArmorStats {
                defense_bonus: 3,
                slot: ArmorSlot::Head,
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn bronze_platebody() -> Self {
        Item {
            name: "Bronze Platebody".to_string(),
            item_type: ItemType::Armor(ArmorStats {
                defense_bonus: 5,
                slot: ArmorSlot::Body,
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn bronze_platelegs() -> Self {
        Item {
            name: "Bronze Platelegs".to_string(),
            item_type: ItemType::Armor(ArmorStats {
                defense_bonus: 4,
                slot: ArmorSlot::Legs,
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn shrimp() -> Self {
        Item {
            name: "Shrimp".to_string(),
            item_type: ItemType::Food(3), // Heals 3 HP
            stackable: false,
            quantity: 1,
        }
    }

    pub fn bronze_axe() -> Self {
        Item {
            name: "Bronze Axe".to_string(),
            item_type: ItemType::Tool(ToolType::Axe {
                woodcutting_level: 1,
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn tinderbox() -> Self {
        Item {
            name: "Tinderbox".to_string(),
            item_type: ItemType::Tool(ToolType::Tinderbox),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn logs() -> Self {
        Item {
            name: "Logs".to_string(),
            item_type: ItemType::Resource(ResourceType::Logs {
                firemaking_level: 1,
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn fishing_rod() -> Self {
        Item {
            name: "Fishing Rod".to_string(),
            item_type: ItemType::Tool(ToolType::FishingRod { fishing_level: 1 }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn bait() -> Self {
        Item {
            name: "Fishing Bait".to_string(),
            item_type: ItemType::Resource(ResourceType::Bait),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn raw_shrimp() -> Self {
        Item {
            name: "Raw Shrimp".to_string(),
            item_type: ItemType::Resource(ResourceType::RawFish { 
                cooking_level: 1, 
                burn_level: 1 
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn raw_trout() -> Self {
        Item {
            name: "Raw Trout".to_string(),
            item_type: ItemType::Resource(ResourceType::RawFish { 
                cooking_level: 15, 
                burn_level: 15 
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn cooked_shrimp() -> Self {
        Item {
            name: "Cooked Shrimp".to_string(),
            item_type: ItemType::Food(3), // Heals 3 HP
            stackable: false,
            quantity: 1,
        }
    }

    pub fn cooked_trout() -> Self {
        Item {
            name: "Cooked Trout".to_string(),
            item_type: ItemType::Food(7), // Heals 7 HP
            stackable: false,
            quantity: 1,
        }
    }

    pub fn cooked_fish() -> Self {
        Self {
            name: "Cooked fish".to_string(),
            item_type: ItemType::Resource(ResourceType::CookedFish { healing: 3 }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn burnt_fish() -> Self {
        Self {
            name: "Burnt fish".to_string(),
            item_type: ItemType::Resource(ResourceType::BurntFish),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn beef() -> Self {
        Item {
            name: "Beef".to_string(),
            item_type: ItemType::Food(4), // Heals 4 HP
            stackable: false,
            quantity: 1,
        }
    }

    pub fn cow_hide() -> Self {
        Item {
            name: "Cow hide".to_string(),
            item_type: ItemType::Resource(ResourceType::Hide),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn bones() -> Self {
        Item {
            name: "Bones".to_string(),
            item_type: ItemType::Resource(ResourceType::Bones),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn raw_beef() -> Self {
        Item {
            name: "Raw beef".to_string(),
            item_type: ItemType::Resource(ResourceType::RawBeef {
                cooking_level: 1,
                burn_level: 30,
            }),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn cooked_beef() -> Self {
        Item {
            name: "Cooked beef".to_string(),
            item_type: ItemType::Food(8), // Heals 8 HP like in RuneScape
            stackable: false,
            quantity: 1,
        }
    }

    pub fn burnt_beef() -> Self {
        Item {
            name: "Burnt beef".to_string(),
            item_type: ItemType::Resource(ResourceType::BurntBeef),
            stackable: false,
            quantity: 1,
        }
    }

    pub fn can_equip(&self) -> bool {
        matches!(self.item_type, ItemType::Weapon(_) | ItemType::Armor(_))
    }

    pub fn is_tool(&self) -> bool {
        matches!(self.item_type, ItemType::Tool(_))
    }

    pub fn is_resource(&self) -> bool {
        matches!(self.item_type, ItemType::Resource(_))
    }

    pub fn use_item(&self, combat: &mut Combat) -> bool {
        match &self.item_type {
            ItemType::Food(healing) => {
                combat.heal(*healing);
                println!("Ate {} and healed {} HP", self.name, healing);
                true // Item was consumed
            }
            _ => false, // Item wasn't consumed
        }
    }

    pub fn is_stackable(&self) -> bool {
        self.stackable || matches!(self.item_type, ItemType::Currency(_))
    }

    pub fn stack_with(&mut self, other: &Item) -> bool {
        if self.name == other.name && self.is_stackable() {
            self.quantity += other.quantity;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<Option<Item>>,
    capacity: usize,
}

#[derive(Debug)]
pub struct DroppedItem {
    pub item: Item,
    pub x: f32,
    pub y: f32,
    pub pickup_delay: f32,
}

impl DroppedItem {
    pub fn new(item: Item, x: f32, y: f32) -> Self {
        DroppedItem { 
            item, 
            x, 
            y,
            pickup_delay: 1.0 
        }
    }

    pub fn draw_with_offset(&self, canvas: &mut Canvas, offset_x: f32, offset_y: f32, sprites: &SpriteManager) -> GameResult {
        let sprite_name = match &self.item.item_type {
            ItemType::Tool(ToolType::Axe { .. }) => "axe",
            ItemType::Resource(ResourceType::Logs { .. }) => "logs",
            ItemType::Resource(ResourceType::RawFish { .. }) => "fish",
            ItemType::Resource(ResourceType::CookedFish { .. }) => "fish",
            ItemType::Resource(ResourceType::Hide) => "cow_hide",
            ItemType::Resource(ResourceType::Bones) => "bones",
            ItemType::Resource(ResourceType::RawBeef { .. }) => "raw_beef",
            ItemType::Resource(ResourceType::BurntBeef) => "burnt_beef",
            ItemType::Food(_) => {
                if self.item.name.contains("beef") {
                    "cooked_beef"
                } else {
                    "fish" // Default for other food items
                }
            },
            _ => "sword", // Default to sword sprite for unknown items
        };

        if let Some(sprite) = sprites.get_sprite(sprite_name) {
            // Draw at world position minus camera offset
            canvas.draw(
                sprite,
                graphics::DrawParam::new()
                    .dest(Vec2::new(
                        self.x - offset_x - 16.0,
                        self.y - offset_y - 16.0
                    ))
                    .scale(Vec2::new(2.0, 2.0))
            );

            // Draw item name above the sprite
            let text = graphics::Text::new(self.item.name.chars().next().unwrap_or('?').to_string());
            canvas.draw(
                &text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(
                        self.x - offset_x - 4.0,
                        self.y - offset_y - 24.0
                    ))
                    .color(Color::WHITE)
            );
        }

        Ok(())
    }

    pub fn update(&mut self, dt: f32) {
        if self.pickup_delay > 0.0 {
            self.pickup_delay -= dt;
        }
    }

    pub fn can_pickup(&self) -> bool {
        self.pickup_delay <= 0.0
    }

    pub fn draw(&self, canvas: &mut Canvas, sprites: &SpriteManager) -> GameResult {
        self.draw_with_offset(canvas, 0.0, 0.0, sprites)
    }
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        Inventory {
            items: vec![None; capacity],
            capacity,
        }
    }

    pub fn add_item(&mut self, item: Item) -> bool {
        if item.is_stackable() {
            for existing_item in self.items.iter_mut().filter_map(|x| x.as_mut()) {
                if existing_item.name == item.name {
                    existing_item.quantity += item.quantity;
                    return true;
                }
            }
        }

        if let Some(empty_slot) = self.items.iter_mut().find(|slot| slot.is_none()) {
            *empty_slot = Some(item);
            true
        } else {
            false // Inventory is full
        }
    }

    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if let Some(Some(item)) = self.items.get_mut(index) {
            if item.is_stackable() && item.quantity > 1 {
                item.quantity -= 1;
                Some(Item {
                    name: item.name.clone(),
                    item_type: item.item_type.clone(),
                    stackable: item.stackable,
                    quantity: 1,
                })
            } else {
                self.items[index].take()
            }
        } else {
            None
        }
    }

    pub fn remove_items(&mut self, index: usize, amount: u32) -> Option<Item> {
        if let Some(Some(item)) = self.items.get_mut(index) {
            if !item.is_stackable() || amount > item.quantity {
                return None;
            }
            
            if amount == item.quantity {
                self.items[index].take()
            } else {
                item.quantity -= amount;
                Some(Item {
                    name: item.name.clone(),
                    item_type: item.item_type.clone(),
                    stackable: item.stackable,
                    quantity: amount,
                })
            }
        } else {
            None
        }
    }

    pub fn get_items(&self) -> &Vec<Option<Item>> {
        &self.items
    }

    pub fn use_item(&mut self, index: usize, combat: &mut Combat) -> bool {
        if let Some(Some(item)) = self.items.get(index) {
            if item.use_item(combat) {
                // Remove the item if it was consumed
                self.items[index] = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_item(&self, index: usize) -> Option<&Item> {
        if index < self.capacity {
            self.items[index].as_ref()
        } else {
            None
        }
    }
} 