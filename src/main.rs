use ggez::{Context, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};
use ggez::conf::{WindowSetup, WindowMode};
use ggez::glam::Vec2;
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::input::mouse::MouseButton;
use rand::Rng;
use std::time::Duration;

mod skills;
mod ui;
mod combat;
mod entity;
mod inventory;
mod equipment;
mod world;
mod save;
mod sprites;
mod world_objects;
mod bank;

use skills::Skills;
use ui::{GameUI, ContextMenuAction};
use combat::Combat;
use entity::Entity;
use inventory::{Inventory, Item, DroppedItem, ItemType, ToolType, ResourceType, ArmorSlot};
use equipment::Equipment;
use world::{Tree, Fire, FishingSpot, FishType};
use save::{SaveData, create_save_data};
use sprites::SpriteManager;
use world_objects::{WorldObject, ObjectType};
use bank::Bank;
use crate::entity::EntityType;

#[derive(Clone, Debug)]
enum PendingAction {
    ChopTree(usize),
    PickupItem(usize),
    Attack,
    Fish(f32, f32),
    None,
}

#[derive(Clone, Debug)]
enum OngoingAction {
    ChoppingTree { x: f32, y: f32, tree_index: usize },
    Fighting { target_index: usize },
    Fishing { x: f32, y: f32, spot_index: usize },
    None,
}

pub struct GameState {
    player_x: f32,
    player_y: f32,
    camera_x: f32,
    camera_y: f32,
    movement_speed: f32,
    skills: Skills,
    game_ui: GameUI,
    player_combat: Combat,
    entities: Vec<Entity>,
    inventory: Inventory,
    equipment: Equipment,
    dropped_items: Vec<DroppedItem>,
    trees: Vec<Tree>,
    fires: Vec<Fire>,
    fishing_spots: Vec<FishingSpot>,
    fishing_spot_timer: f32,
    last_update: std::time::Instant,
    selected_item: Option<usize>,
    target_x: Option<f32>,
    target_y: Option<f32>,
    pending_action: PendingAction,
    ongoing_action: OngoingAction,
    action_timer: f32,
    sprite_manager: &'static SpriteManager,
    world_objects: Vec<WorldObject>,
    pub bank: Bank,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let sprite_manager = Box::leak(Box::new(SpriteManager::new(ctx)?));
        
        // Try to load saved game
        let mut state = if let Ok(Some(save_data)) = SaveData::load_from_file(ctx) {
            Self {
                player_x: save_data.player_x,
                player_y: save_data.player_y,
                camera_x: 0.0,
                camera_y: 0.0,
                movement_speed: 4.0,
                skills: save_data.skills,
                game_ui: GameUI::new(sprite_manager),
                player_combat: Combat::new(save_data.max_health),
                entities: Vec::new(),
                inventory: save_data.inventory,
                equipment: save_data.equipment,
                dropped_items: Vec::new(),
                trees: Vec::new(),
                fires: Vec::new(),
                fishing_spots: Vec::new(),
                fishing_spot_timer: 0.0,
                last_update: std::time::Instant::now(),
                selected_item: None,
                target_x: None,
                target_y: None,
                pending_action: PendingAction::None,
                ongoing_action: OngoingAction::None,
                action_timer: 0.0,
                sprite_manager: &*sprite_manager,
                world_objects: Vec::new(),
                bank: save_data.bank,
            }
        } else {
            // Create new game state
            Self {
                player_x: 512.0,
                player_y: 384.0,
                camera_x: 0.0,
                camera_y: 0.0,
                movement_speed: 4.0,
                skills: Skills::new(),
                game_ui: GameUI::new(sprite_manager),
                player_combat: Combat::new(20),
                entities: Vec::new(),
                inventory: Inventory::new(28),
                equipment: Equipment::new(),
                dropped_items: Vec::new(),
                trees: Vec::new(),
                fires: Vec::new(),
                fishing_spots: Vec::new(),
                fishing_spot_timer: 0.0,
                last_update: std::time::Instant::now(),
                selected_item: None,
                target_x: None,
                target_y: None,
                pending_action: PendingAction::None,
                ongoing_action: OngoingAction::None,
                action_timer: 0.0,
                sprite_manager: &*sprite_manager,
                world_objects: Vec::new(),
                bank: Bank::new(800),
            }
        };

        // Add starting equipment only for new games
        if state.inventory.get_items().iter().all(|item| item.is_none()) {
            state.inventory.add_item(Item::bronze_sword());
            state.inventory.add_item(Item::bronze_helmet());
            state.inventory.add_item(Item::bronze_platebody());
            state.inventory.add_item(Item::bronze_platelegs());
            state.inventory.add_item(Item::bronze_axe());
            state.inventory.add_item(Item::tinderbox());
            state.inventory.add_item(Item::fishing_rod());
            state.inventory.add_item(Item::gp(1000));
        }

        state.spawn_world_objects();
        Ok(state)
    }

    fn spawn_world_objects(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Spawn bank chests in useful locations
        self.world_objects.push(WorldObject::new(100.0, 100.0, ObjectType::BankChest)); // Near starting area
        self.world_objects.push(WorldObject::new(500.0, 500.0, ObjectType::BankChest)); // Near forest
        
        // Spawn forest areas
        let forest_regions = [
            // Dense forest area
            (700.0, -200.0, 0.7),
            (800.0, -150.0, 0.8),
            (750.0, -100.0, 0.6),
            (850.0, -50.0, 0.7),
            // Additional forest regions
            (300.0, 200.0, 0.6),
            (400.0, 250.0, 0.7),
            (-200.0, -300.0, 0.8),
            (-300.0, -250.0, 0.6)
        ];

        // Create natural tree clusters
        for &(center_x, center_y, density) in &forest_regions {
            for dx in -3..=3 {
                for dy in -3..=3 {
                    let x = center_x + dx as f32 * 80.0 + rng.gen_range(-20.0..20.0);
                    let y = center_y + dy as f32 * 80.0 + rng.gen_range(-20.0..20.0);
                    
                    // Higher chance of trees near center and based on density
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    let prob = (density * (1.0 - distance / 4.0)).max(0.1) as f64;
                    if rng.gen_bool(prob) {
                        self.world_objects.push(WorldObject::new(x, y, ObjectType::Tree));
                    }
                }
            }
        }

        // Spawn goblin camps
        let goblin_camps = [
            (100.0, -100.0, 4),  // Small camp
            (-150.0, 200.0, 3),  // Another small camp
            (400.0, 300.0, 5)    // Larger camp
        ];

        for &(center_x, center_y, count) in &goblin_camps {
            // Spawn goblins in a loose group
            for _ in 0..count {
                let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                let distance = rng.gen_range(0.0..80.0);
                let x = center_x + angle.cos() * distance;
                let y = center_y + angle.sin() * distance;
                self.entities.push(Entity::new_goblin(x, y));
            }
        }

        // Spawn cow herds
        let cow_pastures = [
            (0.0, 0.0, 5),      // Central pasture
            (-300.0, 100.0, 3),  // Western pasture
            (200.0, -200.0, 4)   // Northern pasture
        ];

        for &(center_x, center_y, count) in &cow_pastures {
            // Spawn cows in a loose group
            for _ in 0..count {
                let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                let distance = rng.gen_range(0.0..100.0);
                let x = center_x + angle.cos() * distance;
                let y = center_y + angle.sin() * distance;
                self.entities.push(Entity::new_cow(x, y));
            }
        }
    }

    fn check_collision(&self, x: f32, y: f32) -> bool {
        const PLAYER_SIZE: f32 = 32.0;
        
        // Only check collision with world objects, not entities
        for obj in &self.world_objects {
            if obj.collides_with(x, y, PLAYER_SIZE, PLAYER_SIZE) {
                return true;
            }
        }
        false
    }

    fn update_movement(&mut self, dt: f32) {
        if let (Some(target_x), Some(target_y)) = (self.target_x, self.target_y) {
            let dx = target_x - self.player_x;
            let dy = target_y - self.player_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 40.0 {
                let speed = self.movement_speed * dt * 60.0;
                let move_x = (dx / distance) * speed;
                let move_y = (dy / distance) * speed;

                // Check collision before moving
                let new_x = (self.player_x + move_x).max(-512.0).min(2048.0);
                let new_y = (self.player_y + move_y).max(-512.0).min(2048.0);

                // Try moving in both directions independently
                if !self.check_collision(new_x, self.player_y) {
                    self.player_x = new_x;
                }
                if !self.check_collision(self.player_x, new_y) {
                    self.player_y = new_y;
                }
            } else {
                self.target_x = None;
                self.target_y = None;
                
                match self.pending_action.clone() {
                    PendingAction::PickupItem(index) => {
                        if index < self.dropped_items.len() {
                            let dropped_item = &self.dropped_items[index];
                            if self.inventory.add_item(dropped_item.item.clone()) {
                                self.game_ui.add_message(format!("You pick up the {}.", dropped_item.item.name));
                                self.dropped_items.remove(index);
                            } else {
                                self.game_ui.add_message("Your inventory is full.".to_string());
                            }
                        }
                    }
                    PendingAction::ChopTree(tree_index) => {
                        if tree_index < self.world_objects.len() {
                            let tree = &self.world_objects[tree_index];
                            if !tree.fallen {
                            self.ongoing_action = OngoingAction::ChoppingTree { 
                                x: tree.x, 
                                y: tree.y, 
                                tree_index 
                            };
                            self.action_timer = 0.0;
                                self.game_ui.add_message("You begin chopping the tree.".to_string());
                            } else {
                                self.game_ui.add_message("This tree is already chopped down.".to_string());
                            }
                        }
                    }
                    PendingAction::Attack => {
                        if let Some((index, _)) = self.entities.iter().enumerate()
                            .find(|(_, e)| e.is_near(self.player_x, self.player_y) && e.is_alive())
                        {
                            self.ongoing_action = OngoingAction::Fighting { target_index: index };
                            self.action_timer = 0.0;
                        }
                    }
                    PendingAction::Fish(x, y) => {
                        if let Some((index, _)) = self.fishing_spots.iter().enumerate()
                            .find(|(_, s)| s.is_near(x, y))
                        {
                            self.ongoing_action = OngoingAction::Fishing { x, y, spot_index: index };
                            self.action_timer = 0.0;
                        }
                    }
                    PendingAction::None => {}
                }
                self.pending_action = PendingAction::None;
            }
        }
    }

    fn save_game(&mut self, ctx: &Context) {
        let save_data = create_save_data(
            self.player_x,
            self.player_y,
            &self.skills,
            &self.player_combat,
            &self.inventory,
            &self.equipment,
            &self.bank,
        );

        match save_data.save_to_file(ctx) {
            Ok(_) => self.game_ui.add_message("Game saved successfully!".to_string()),
            Err(e) => self.game_ui.add_message(format!("Error saving game: {}", e)),
        }
    }

    fn set_destination(&mut self, x: f32, y: f32, action: PendingAction) {
        println!("Debug: Setting destination to ({}, {}) with action: {:?}", x, y, action);
        match &action {
            PendingAction::ChopTree(index) => {
                println!("Debug: Setting ChopTree action for tree index {}", index);
            }
            _ => {}
        }
        self.target_x = Some(x);
        self.target_y = Some(y);
        self.pending_action = action;
    }

    fn is_near_target(&self) -> bool {
        if let (Some(target_x), Some(target_y)) = (self.target_x, self.target_y) {
            let dx = target_x - self.player_x;
            let dy = target_y - self.player_y;
            let distance = (dx * dx + dy * dy).sqrt();
            distance < 40.0
        } else {
            false
        }
    }

    fn cancel_ongoing_action(&mut self) {
        self.ongoing_action = OngoingAction::None;
        self.action_timer = 0.0;
    }

    fn update_ongoing_action(&mut self, dt: f32) {
        self.action_timer -= dt;
        
        if self.action_timer <= 0.0 {
            match &self.ongoing_action.clone() {
                OngoingAction::ChoppingTree { x: _, y: _, tree_index } => {
                    println!("Debug: Attempting to chop tree at index {}", tree_index);
                    if let Some(tree) = self.world_objects.get_mut(*tree_index) {
                        // Check distance to tree
                        let dx = tree.x - self.player_x;
                        let dy = tree.y - self.player_y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        
                        println!("Debug: Distance to tree: {}", distance);
                        if distance > 40.0 {
                            println!("Debug: Too far from tree, moving closer");
                            let tree_x = tree.x;
                            let tree_y = tree.y;
                            self.set_destination(tree_x, tree_y, PendingAction::ChopTree(*tree_index));
                            return;
                        }

                        println!("Debug: Attempting to chop tree. Tree health: {}, Tree fallen: {}", tree.health, tree.fallen);
                        
                        // Find axe in inventory and get its woodcutting level
                        let axe_info = {
                            let items = self.inventory.get_items();
                            items.iter()
                                .find(|item| {
                                    if let Some(item) = item {
                                        matches!(&item.item_type, ItemType::Tool(ToolType::Axe { .. }))
                                    } else {
                                        false
                                    }
                                })
                                .and_then(|item| item.as_ref())
                                .map(|axe| {
                                    if let ItemType::Tool(ToolType::Axe { woodcutting_level }) = axe.item_type {
                                        (axe.clone(), woodcutting_level)
                                    } else {
                                        unreachable!()
                                    }
                                })
                        };

                        if axe_info.is_none() {
                            self.game_ui.add_message("You need an axe to chop trees.".to_string());
                            self.cancel_ongoing_action();
                            return;
                        }

                        let (axe, woodcutting_level) = axe_info.unwrap();
                        if tree.try_chop(&self.skills, Some(&axe)) {
                            println!("Debug: Successfully chopped tree");
                            self.game_ui.add_message("You swing your axe at the tree.".to_string());
                            
                            // Add one log to inventory
                            if self.inventory.add_item(Item::logs()) {
                                println!("Debug: Added 1 log to inventory");
                                self.skills.gain_woodcutting_xp(25);
                                self.game_ui.add_message("You get a log.".to_string());
                            } else {
                                self.game_ui.add_message("Your inventory is full.".to_string());
                                self.cancel_ongoing_action();
                                return;
                            }

                            if tree.fallen {
                                println!("Debug: Tree is now fully chopped");
                                self.game_ui.add_message("The tree falls down!".to_string());
                                self.cancel_ongoing_action();
                        } else {
                                println!("Debug: Setting next chop timer");
                                // Calculate chop time based on woodcutting level and axe type
                                let base_time = 3.0;
                                let level_bonus = self.skills.woodcutting.get_level() as f32 * 0.03;
                                let axe_bonus = woodcutting_level as f32 * 0.05;
                                self.action_timer = (base_time - level_bonus - axe_bonus).max(1.2);
                            }
                        } else {
                            println!("Debug: Failed to chop tree");
                            if tree.is_chopped() || tree.fallen {
                                self.game_ui.add_message("This tree is already chopped down.".to_string());
                            } else {
                                if let ItemType::Tool(ToolType::Axe { woodcutting_level }) = &axe.item_type {
                                    println!("Debug: Player lacks required woodcutting level {}", woodcutting_level);
                                    self.game_ui.add_message(format!("You need level {} Woodcutting to use this axe.", woodcutting_level));
                                }
                            }
                            self.cancel_ongoing_action();
                        }
                    } else {
                        println!("Debug: Tree index {} not found", tree_index);
                        self.cancel_ongoing_action();
                    }
                }
                OngoingAction::Fighting { target_index } => {
                    if *target_index < self.entities.len() {
                        let target = &self.entities[*target_index];
                        if !target.is_alive() {
                            self.cancel_ongoing_action();
                            return;
                        }

                        if !target.is_near(self.player_x, self.player_y) {
                            let target_pos = target.get_position();
                            self.set_destination(target_pos.0, target_pos.1, PendingAction::Attack);
                            return;
                        }

                        self.attack_nearest_entity();
                        self.action_timer = 2.4;
                    } else {
                        self.cancel_ongoing_action();
                    }
                }
                OngoingAction::Fishing { x, y, spot_index } => {
                    if *spot_index < self.fishing_spots.len() {
                        let spot = &self.fishing_spots[*spot_index];
                        
                        let dx = *x - self.player_x;
                        let dy = *y - self.player_y;
                        if (dx * dx + dy * dy).sqrt() > 40.0 {
                            self.set_destination(*x, *y, PendingAction::Fish(*x, *y));
                            return;
                        }

                        let rod = self.inventory.get_items().iter()
                            .filter_map(|item| item.as_ref())
                            .find(|item| matches!(&item.item_type, ItemType::Tool(ToolType::FishingRod { .. })));
                        
                        let has_bait = self.inventory.get_items().iter()
                            .filter_map(|item| item.as_ref())
                            .any(|item| matches!(&item.item_type, ItemType::Resource(ResourceType::Bait)));

                        if let Some(fish) = spot.try_fish(&self.skills, rod, has_bait) {
                            if matches!(spot.fish_type, FishType::Trout) {
                                if let Some(bait_slot) = self.inventory.get_items().iter()
                                    .enumerate()
                                    .filter_map(|(i, item)| item.as_ref().map(|it| (i, it)))
                                    .find(|(_, item)| matches!(&item.item_type, ItemType::Resource(ResourceType::Bait)))
                                    .map(|(i, _)| i)
                                {
                                    self.inventory.remove_item(bait_slot);
                                }
                            }

                            if self.inventory.add_item(fish.clone()) {
                                self.game_ui.add_message(format!("You catch a {}.", fish.name));
                                self.skills.gain_fishing_xp(match spot.fish_type {
                                    FishType::Shrimp => 10,
                                    FishType::Trout => 50,
                                });
                                self.action_timer = 3.0;
                            } else {
                                self.game_ui.add_message("Your inventory is full.".to_string());
                                self.cancel_ongoing_action();
                            }
                        } else {
                            self.game_ui.add_message("You fail to catch anything.".to_string());
                            self.action_timer = 3.0;
                        }
                    } else {
                        self.cancel_ongoing_action();
                    }
                }
                OngoingAction::None => {}
            }
        }
    }

    fn attack_nearest_entity(&mut self) {
        if let Some(target_index) = self.entities.iter()
            .enumerate()
            .filter(|(_, e)| e.is_near(self.player_x, self.player_y))
            .find(|(_, e)| e.is_alive())
            .map(|(i, _)| i)
        {
            let target = &mut self.entities[target_index];
                let attack_bonus = self.equipment.get_total_attack_bonus();
                let strength_bonus = self.equipment.get_total_strength_bonus();
                let defense_bonus = self.equipment.get_total_defense_bonus();

            // Get target name first
            let target_name = match &target.entity_type {
                EntityType::Goblin(_) => "goblin",
                EntityType::Cow(_) => "cow",
            };

            let (target_x, target_y) = target.get_position();

            if let Some(target_combat) = target.get_combat_mut() {
                if let Some(damage) = self.player_combat.attack(&self.skills, &Skills::new(), attack_bonus, strength_bonus, 0) {
                    self.game_ui.add_message(format!("You attack the {}!", target_name.chars().next().unwrap().to_uppercase().collect::<String>() + &target_name[1..]));
                    target_combat.take_damage(damage as i32);
                    self.skills.gain_attack_xp(4);
                    
                    if target_combat.is_dead() {
                        self.game_ui.add_message(format!("The {} is dead!", target_name.chars().next().unwrap().to_uppercase().collect::<String>() + &target_name[1..]));
                        let drops = target.get_drops();
                            for item in drops {
                            self.dropped_items.push(DroppedItem::new(item, target_x, target_y));
                        }
                        self.skills.gain_attack_xp(10);
                        self.skills.gain_strength_xp(10);
                        self.skills.gain_defense_xp(10);
                    } else {
                        if let Some(damage) = target_combat.attack(&Skills::new(), &self.skills, 0, 0, defense_bonus) {
                            self.game_ui.add_message(format!("You hit the {} for {} damage!", target_name.chars().next().unwrap().to_uppercase().collect::<String>() + &target_name[1..], damage));
                            self.player_combat.take_damage(damage as i32);
                            self.skills.gain_defense_xp(4);
                        } else {
                            self.game_ui.add_message(format!("{} misses!", target_name.chars().next().unwrap().to_uppercase().collect::<String>() + &target_name[1..]));
                        }
                    }
                } else {
                    self.game_ui.add_message("Player misses!".to_string());
                }
            }
        }
    }

    fn handle_inventory_click(&mut self, slot: usize, button: MouseButton) {
        if let Some(item) = self.inventory.get_item(slot) {
            match button {
                MouseButton::Left => {
                    if let Some(selected_slot) = self.selected_item {
                        if let Some(selected_item) = self.inventory.get_item(selected_slot) {
                            match (&selected_item.item_type, &item.item_type) {
                                (ItemType::Tool(ToolType::Tinderbox), ItemType::Resource(ResourceType::Logs { firemaking_level })) |
                                (ItemType::Resource(ResourceType::Logs { firemaking_level }), ItemType::Tool(ToolType::Tinderbox)) => {
                                    if u32::from(self.skills.firemaking.get_level()) >= *firemaking_level {
                                        self.fires.push(Fire::new(self.player_x, self.player_y));
                                        let logs_slot = if matches!(selected_item.item_type, ItemType::Tool(ToolType::Tinderbox)) {
                                            slot
                                        } else {
                                            selected_slot
                                        };
                                        if self.inventory.remove_item(logs_slot).is_some() {
                                            self.skills.gain_firemaking_xp(40);
                                            self.game_ui.add_message("You light a fire.".to_string());
                                        }
                                    } else {
                                        self.game_ui.add_message(format!("You need level {} Firemaking to light these logs.", firemaking_level));
                                    }
                                }
                                _ => {
                                    let item_clone = item.clone();
                                    if let ItemType::Resource(ResourceType::RawFish { cooking_level, .. }) = &item_clone.item_type {
                                        if let Some(fire) = self.fires.iter()
                                            .find(|f| f.is_near(self.player_x, self.player_y))
                                        {
                                            if u32::from(self.skills.cooking.get_level()) >= *cooking_level {
                                                if let Some(cooked_item) = fire.try_cook(&item_clone, self.skills.cooking.get_level()) {
                                                    self.inventory.remove_item(slot);
                                                    if self.inventory.add_item(cooked_item.clone()) {
                                                        match cooked_item.name.as_str() {
                                                            "Burnt fish" => self.game_ui.add_message("You accidentally burn the fish.".to_string()),
                                                            "Burnt beef" => self.game_ui.add_message("You accidentally burn the beef.".to_string()),
                                                            _ => {
                                                                self.game_ui.add_message(format!("You successfully cook the {}.", item_clone.name.strip_prefix("Raw ").unwrap_or(&item_clone.name)));
                                                                self.skills.gain_cooking_xp(30);
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                self.game_ui.add_message(format!("You need level {} Cooking to cook this.", cooking_level));
                                            }
                                        } else {
                                            self.game_ui.add_message("You need to be near a fire to cook food.".to_string());
                                        }
                                    }
                                    if let ItemType::Resource(ResourceType::RawBeef { cooking_level, .. }) = &item_clone.item_type {
                                        if let Some(fire) = self.fires.iter()
                                            .find(|f| f.is_near(self.player_x, self.player_y))
                                        {
                                            if u32::from(self.skills.cooking.get_level()) >= *cooking_level {
                                                if let Some(cooked_item) = fire.try_cook(&item_clone, self.skills.cooking.get_level()) {
                                                    self.inventory.remove_item(slot);
                                                    if self.inventory.add_item(cooked_item.clone()) {
                                                        match cooked_item.name.as_str() {
                                                            "Burnt beef" => self.game_ui.add_message("You accidentally burn the beef.".to_string()),
                                                            _ => {
                                                                self.game_ui.add_message(format!("You successfully cook the {}.", item_clone.name.strip_prefix("Raw ").unwrap_or(&item_clone.name)));
                                                                self.skills.gain_cooking_xp(30);
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                self.game_ui.add_message(format!("You need level {} Cooking to cook this.", cooking_level));
                                            }
                                        } else {
                                            self.game_ui.add_message("You need to be near a fire to cook food.".to_string());
                                        }
                                    }
                                }
                            }
                        }
                        self.selected_item = None;
                        self.game_ui.clear_selection();
                    } else {
                        // Handle equipment when clicking directly on an item
                        match &item.item_type {
                            ItemType::Weapon(_) | ItemType::Armor(_) => {
                                if item.can_equip() {
                                    if let Some(item) = self.inventory.remove_item(slot) {
                                        let item_name = item.name.clone();
                                        let old_item = match &item.item_type {
                                            ItemType::Weapon(_) => self.equipment.equip_weapon(item),
                                            ItemType::Armor(_) => self.equipment.equip_armor(item),
                                            _ => None,
                                        };
                                        
                                        if let Some(old_item) = old_item {
                                            self.inventory.add_item(old_item);
                                        }
                                        self.game_ui.add_message(format!("Equipped {}", item_name));
                                    }
                                } else {
                                    self.game_ui.add_message("You cannot equip this item.".to_string());
                                }
                            }
                            ItemType::Food(_) => {
                                self.inventory.use_item(slot, &mut self.player_combat);
                            }
                            _ => {
                                self.selected_item = Some(slot);
                                self.game_ui.select_slot(slot);
                            }
                        }
                    }
                }
                MouseButton::Right => self.drop_item(slot),
                _ => {}
            }
        }
    }

    fn drop_item(&mut self, slot: usize) {
        if let Some(item) = self.inventory.remove_item(slot) {
            self.dropped_items.push(DroppedItem::new(
                item,
                self.player_x,
                self.player_y,
            ));
        }
    }

    fn try_chop_tree(&mut self) {
        if let Some((index, tree)) = self.world_objects.iter().enumerate()
            .find(|(_, obj)| {
                let dx = obj.x - self.player_x;
                let dy = obj.y - self.player_y;
                (dx * dx + dy * dy).sqrt() < 40.0 && matches!(obj.object_type, ObjectType::Tree)
            })
        {
            let axe = self.inventory.get_items().iter()
                .filter_map(|item| item.as_ref())
                .find(|item| matches!(&item.item_type, ItemType::Tool(ToolType::Axe { .. })));

            if let Some(axe) = axe {
                if let ItemType::Tool(ToolType::Axe { woodcutting_level }) = &axe.item_type {
                    if u32::from(self.skills.woodcutting.get_level()) >= *woodcutting_level {
                        if self.inventory.add_item(Item::logs()) {
                            self.game_ui.add_message("You get some logs.".to_string());
                            self.skills.gain_woodcutting_xp(25);
                        } else {
                            self.game_ui.add_message("Your inventory is full.".to_string());
                        }
                    } else {
                        self.game_ui.add_message(format!("You need level {} Woodcutting to use this axe.", woodcutting_level));
                    }
                }
            } else {
                self.game_ui.add_message("You need an axe to chop trees.".to_string());
            }
        }
    }

    fn handle_world_click(&mut self, screen_x: f32, screen_y: f32, button: MouseButton) {
        // Convert screen coordinates to world coordinates by adding camera offset
        let world_x = screen_x + self.camera_x;
        let world_y = screen_y + self.camera_y;

        // If the context menu is visible and we click outside it, hide it
        if self.game_ui.context_menu.visible {
            if let Some(action) = self.game_ui.context_menu.handle_click(screen_x, screen_y) {
                self.handle_context_action(action, world_x, world_y);
            }
            self.game_ui.context_menu.hide();
            return;
        }

        // Cancel ongoing action when clicking elsewhere
        if button == MouseButton::Left {
            self.cancel_ongoing_action();
        }

        if button == MouseButton::Right {
            let mut actions = Vec::new();
            
            // Check for nearby world objects
            for obj in &self.world_objects {
                let dx = obj.x - world_x;
                let dy = obj.y - world_y;
                if (dx * dx + dy * dy).sqrt() < 40.0 {
                    match obj.object_type {
                        ObjectType::Tree => {
                            if !obj.fallen {
                            actions.push(("Chop tree".to_string(), ContextMenuAction::ChopTree));
                            }
                            actions.push(("Examine tree".to_string(), ContextMenuAction::Examine(
                                if obj.fallen {
                                    "A tree stump. It will regrow soon.".to_string()
                                } else {
                                    "A sturdy tree good for woodcutting.".to_string()
                                }
                            )));
                        }
                        ObjectType::Water => {
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("Clear blue water.".to_string())));
                        }
                        ObjectType::Wall | ObjectType::CastleWall => {
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("A solid stone wall.".to_string())));
                        }
                        ObjectType::CastleDoor => {
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("A heavy wooden door.".to_string())));
                        }
                        ObjectType::CastleStairs => {
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("Stone stairs leading up.".to_string())));
                        }
                        ObjectType::Bridge => {
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("A wooden bridge crossing the river.".to_string())));
                        }
                        ObjectType::Road => {
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("A well-traveled dirt road.".to_string())));
                        }
                        ObjectType::Path => {
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("A narrow dirt path.".to_string())));
                        }
                        ObjectType::Fence => {
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("A wooden fence.".to_string())));
                        }
                        ObjectType::BankChest => {
                            actions.push(("Use Bank".to_string(), ContextMenuAction::OpenBank));
                            actions.push(("Examine".to_string(), ContextMenuAction::Examine("A secure chest for storing your items.".to_string())));
                        }
                    }
                    break; // Only show options for the first object found
                }
            }

            // Check for nearby dropped items
            if let Some((item_index, item)) = self.dropped_items.iter().enumerate()
                .find(|(_, i)| {
                    let dx = i.x - world_x;
                    let dy = i.y - world_y;
                    (dx * dx + dy * dy).sqrt() < 40.0
                })
            {
                actions.push(("Pick up".to_string(), ContextMenuAction::PickupItem));
                actions.push(("Examine".to_string(), ContextMenuAction::Examine(format!("It's a {}.", item.item.name))));
            }

            // Check for nearby goblins or cows
            if let Some(entity) = self.entities.iter()
                .find(|e| e.is_near(world_x, world_y) && e.is_alive())
            {
                actions.push(("Attack".to_string(), ContextMenuAction::Attack));
                let examine_text = match &entity.entity_type {
                    EntityType::Goblin(_) => "A mean-looking goblin.",
                    EntityType::Cow(_) => "A peaceful cow grazing in the field.",
                };
                actions.push(("Examine".to_string(), ContextMenuAction::Examine(examine_text.to_string())));
            }

            // Check for nearby fishing spots
            if let Some(spot) = self.fishing_spots.iter()
                .find(|s| s.is_near(world_x, world_y))
            {
                let action_name = match spot.fish_type {
                    FishType::Shrimp => "Fish for shrimp",
                    FishType::Trout => "Fish for trout",
                };
                actions.push((action_name.to_string(), ContextMenuAction::Fish));
                actions.push(("Examine".to_string(), ContextMenuAction::Examine("A good spot for fishing.".to_string())));
            }

            if !actions.is_empty() {
                self.game_ui.context_menu.show(screen_x, screen_y, actions);
            } else {
                // If no interactions available, just walk there
                self.set_destination(world_x, world_y, PendingAction::None);
            }
        } else if button == MouseButton::Left {
            // Just walk to the clicked location
            self.set_destination(world_x, world_y, PendingAction::None);
        }
    }

    fn handle_context_action(&mut self, action: ContextMenuAction, x: f32, y: f32) {
        match action {
            ContextMenuAction::ChopTree => {
                println!("Debug: ChopTree action received at coordinates ({}, {})", x, y);
                // First find the closest tree to the click location
                let closest_tree = self.world_objects.iter().enumerate()
                    .filter(|(_, obj)| matches!(obj.object_type, ObjectType::Tree) && !obj.fallen)
                    .map(|(i, obj)| {
                        let dx = obj.x - x;
                        let dy = obj.y - y;
                        let dist = dx * dx + dy * dy;
                        println!("Debug: Checking tree at index {} - distance: {}, fallen: {}", i, dist.sqrt(), obj.fallen);
                        (i, obj, dist)
                    })
                    .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

                if let Some((tree_index, tree, dist)) = closest_tree {
                    if dist.sqrt() < 100.0 {
                        println!("Debug: Found valid tree at index {}, distance {}, setting destination", tree_index, dist.sqrt());
                        self.game_ui.add_message("You walk towards the tree...".to_string());
                        self.set_destination(tree.x, tree.y, PendingAction::ChopTree(tree_index));
                    } else {
                        println!("Debug: Closest tree too far away (distance: {})", dist.sqrt());
                    }
                } else {
                    println!("Debug: No valid tree found near click location");
                }
            }
            ContextMenuAction::PickupItem => {
                if let Some((item_index, _)) = self.dropped_items.iter().enumerate()
                    .find(|(_, item)| {
                        let dx = item.x - x;
                        let dy = item.y - y;
                        dx * dx + dy * dy < 1600.0  // 40 unit radius squared
                    }) {
                    self.set_destination(x, y, PendingAction::PickupItem(item_index));
                    self.game_ui.add_message("Walking to pick up the item...".to_string());
                }
            }
            ContextMenuAction::Attack => {
                self.set_destination(x, y, PendingAction::Attack);
            }
            ContextMenuAction::Fish => {
                self.set_destination(x, y, PendingAction::Fish(x, y));
            }
            ContextMenuAction::OpenBank => {
                self.game_ui.toggle_bank();
            }
            ContextMenuAction::Examine(text) => {
                self.game_ui.add_message(text);
            }
            // Handle bank-related actions by delegating to GameUI
            ContextMenuAction::WithdrawOne | 
            ContextMenuAction::WithdrawTen | 
            ContextMenuAction::WithdrawHundred |
            ContextMenuAction::WithdrawAll |
            ContextMenuAction::WithdrawX |
            ContextMenuAction::DepositOne |
            ContextMenuAction::DepositTen |
            ContextMenuAction::DepositHundred |
            ContextMenuAction::DepositX |
            ContextMenuAction::DepositAll => {
                self.game_ui.handle_context_action(action, &mut self.inventory, &mut self.bank);
            }
            ContextMenuAction::None => {}
        }
    }

    fn spawn_fishing_spot(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Define pond area (bottom left of map)
        let x = rng.gen_range(100.0..300.0);
        let y = rng.gen_range(500.0..700.0);
        
        // 70% chance for shrimp spot, 30% for trout
        let fish_type = if rng.gen_bool(0.7) {
            FishType::Shrimp
        } else {
            FishType::Trout
        };
        
        self.fishing_spots.push(FishingSpot::new(x, y, fish_type));
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        // Draw world objects with camera offset
        for obj in &self.world_objects {
            obj.draw(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw trees with camera offset
        for tree in self.trees.iter() {
            tree.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw fires with camera offset
        for fire in self.fires.iter() {
            fire.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw fishing spots with camera offset
        for spot in self.fishing_spots.iter() {
            spot.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw entities with camera offset
        for entity in self.entities.iter() {
            entity.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw dropped items with camera offset
        for item in self.dropped_items.iter() {
            item.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw player
        if let Some(player_sprite) = self.sprite_manager.get_sprite("player") {
            canvas.draw(
                player_sprite,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.player_x - self.camera_x - 16.0, self.player_y - self.camera_y - 16.0))
                    .scale(Vec2::new(2.0, 2.0))
            );
        }

        // Draw player health bar
        let health_percent = self.player_combat.health as f32 / self.player_combat.max_health as f32;
        
        // Black background
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(self.player_x - self.camera_x - 16.0, self.player_y - self.camera_y - 26.0))
                .scale(Vec2::new(32.0, 5.0))
                .color(Color::BLACK)
        );

        // Green health bar
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(self.player_x - self.camera_x - 16.0, self.player_y - self.camera_y - 26.0))
                .scale(Vec2::new(32.0 * health_percent, 5.0))
                .color(Color::GREEN)
        );

        // Draw UI
        (&mut self.game_ui).draw(
            &mut canvas,
            &self.skills,
            &self.inventory,
            &self.equipment,
            &self.bank,
            self.player_x,
            self.player_y,
        )?;

        canvas.finish(ctx)?;
        Ok(())
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Update camera to follow player
        self.camera_x = self.player_x - 512.0; // Half the window width
        self.camera_y = self.player_y - 384.0; // Half the window height

        // Update entities and remove dead ones
        self.entities.retain_mut(|entity| {
            entity.update(dt);
            entity.is_alive()
        });

        // Update trees
        for tree in &mut self.trees {
            tree.update(dt);
        }

        // Update and remove expired fires
        self.fires.retain_mut(|fire| {
            fire.update(dt);
            !fire.is_expired()
        });

        // Update dropped items
        for item in &mut self.dropped_items {
            item.update(dt);
        }

        // Update fishing spot timer
        self.fishing_spot_timer -= dt;
        if self.fishing_spot_timer <= 0.0 {
            self.spawn_fishing_spot();
            self.fishing_spot_timer = 10.0; // Spawn new spot every 10 seconds
        }

        // Update and remove expired fishing spots
        self.fishing_spots.retain_mut(|spot| spot.update(dt));

        // Update movement and actions
        self.update_movement(dt);
        self.update_ongoing_action(dt);

        // Close bank if player moves away from chest
        if self.game_ui.bank_visible {
            let near_bank = self.world_objects.iter()
                .any(|obj| matches!(obj.object_type, ObjectType::BankChest) && {
                    let dx = obj.x - self.player_x;
                    let dy = obj.y - self.player_y;
                    (dx * dx + dy * dy).sqrt() < 40.0
                });

            if !near_bank {
                self.game_ui.bank_visible = false;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        // Draw world objects with camera offset
        for obj in &self.world_objects {
            obj.draw(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw trees with camera offset
        for tree in self.trees.iter() {
            tree.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw fires with camera offset
        for fire in self.fires.iter() {
            fire.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw fishing spots with camera offset
        for spot in self.fishing_spots.iter() {
            spot.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw entities with camera offset
        for entity in self.entities.iter() {
            entity.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw dropped items with camera offset
        for item in self.dropped_items.iter() {
            item.draw_with_offset(&mut canvas, self.camera_x, self.camera_y, &self.sprite_manager)?;
        }

        // Draw player
        if let Some(player_sprite) = self.sprite_manager.get_sprite("player") {
            canvas.draw(
                player_sprite,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.player_x - self.camera_x - 16.0, self.player_y - self.camera_y - 16.0))
                    .scale(Vec2::new(2.0, 2.0))
            );
        }

        // Draw player health bar
        let health_percent = self.player_combat.health as f32 / self.player_combat.max_health as f32;
        
        // Black background
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(self.player_x - self.camera_x - 16.0, self.player_y - self.camera_y - 26.0))
                .scale(Vec2::new(32.0, 5.0))
                .color(Color::BLACK)
        );

        // Green health bar
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(self.player_x - self.camera_x - 16.0, self.player_y - self.camera_y - 26.0))
                .scale(Vec2::new(32.0 * health_percent, 5.0))
                .color(Color::GREEN)
        );

        // Draw UI
        (&mut self.game_ui).draw(
            &mut canvas,
            &self.skills,
            &self.inventory,
            &self.equipment,
            &self.bank,
            self.player_x,
            self.player_y,
        )?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> GameResult {
        // First check if we're clicking in the menu bar
        if self.game_ui.handle_menu_click(x, y) {
            return Ok(());
        }

        // Handle context menu clicks first
        if self.game_ui.context_menu.visible {
            if let Some(action) = self.game_ui.context_menu.handle_click(x, y) {
                let world_x = x + self.camera_x;
                let world_y = y + self.camera_y;
                self.handle_context_action(action, world_x, world_y);
            }
            self.game_ui.context_menu.hide();
            return Ok(());
        }

        // Check if bank is visible and handle bank clicks
        if self.game_ui.bank_visible {
            if self.game_ui.handle_bank_click(x, y, button, &mut self.inventory, &mut self.bank) {
                return Ok(());
            }
        }

        if self.game_ui.inventory_visible {
            // Check if click is in inventory area
            if x >= 30.0 && x <= 210.0 && y >= 50.0 && y <= 365.0 {
                    let slot_x = ((x - 30.0) / 45.0).floor() as usize;
                let slot_y = ((y - 50.0) / 45.0).floor() as usize;
                let slot = slot_y * 4 + slot_x;
                
                if slot < self.inventory.get_items().len() {
                    if self.game_ui.bank_visible {
                        // Handle bank deposit
                        if let Some(item) = self.inventory.get_item(slot).cloned() {
                            // For all items, show deposit options
                            self.game_ui.handle_inventory_click(slot, button, x, y, &mut self.inventory);
                        }
                    } else {
                        self.handle_inventory_click(slot, button);
                    }
                }
            } else if !self.game_ui.bank_visible {
                // Only handle world clicks if bank is not visible
                self.handle_world_click(x, y, button);
            }
        } else if !self.game_ui.is_menu_visible() && !self.game_ui.bank_visible {
            // Only handle world clicks if no menu is visible and bank is not visible
            self.handle_world_click(x, y, button);
        }

        Ok(())
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) -> GameResult {
        if button == MouseButton::Left && !self.game_ui.inventory_visible && !self.game_ui.is_menu_visible() {
            self.selected_item = None;
            self.game_ui.clear_selection();
        }
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::I) => self.game_ui.toggle_inventory(),
            Some(KeyCode::K) => self.game_ui.toggle_skills_menu(),
            Some(KeyCode::E) => self.game_ui.toggle_equipment_screen(),
            Some(KeyCode::S) => self.save_game(ctx),
            Some(KeyCode::Escape) => {
                if self.game_ui.quantity_dialog_visible {
                    self.game_ui.hide_quantity_dialog();
                } else if self.game_ui.bank_visible {
                    self.game_ui.toggle_bank();
                }
            }
            Some(KeyCode::Return) | Some(KeyCode::NumpadEnter) => {
                if self.game_ui.quantity_dialog_visible {
                    self.game_ui.handle_quantity_enter(&mut self.inventory, &mut self.bank);
                }
            }
            Some(KeyCode::Back) => {
                if self.game_ui.quantity_dialog_visible {
                    self.game_ui.handle_quantity_backspace();
                }
            }
            Some(key) => {
                if self.game_ui.quantity_dialog_visible {
                    match key {
                        KeyCode::Key0 | KeyCode::Numpad0 => self.game_ui.handle_quantity_input('0'),
                        KeyCode::Key1 | KeyCode::Numpad1 => self.game_ui.handle_quantity_input('1'),
                        KeyCode::Key2 | KeyCode::Numpad2 => self.game_ui.handle_quantity_input('2'),
                        KeyCode::Key3 | KeyCode::Numpad3 => self.game_ui.handle_quantity_input('3'),
                        KeyCode::Key4 | KeyCode::Numpad4 => self.game_ui.handle_quantity_input('4'),
                        KeyCode::Key5 | KeyCode::Numpad5 => self.game_ui.handle_quantity_input('5'),
                        KeyCode::Key6 | KeyCode::Numpad6 => self.game_ui.handle_quantity_input('6'),
                        KeyCode::Key7 | KeyCode::Numpad7 => self.game_ui.handle_quantity_input('7'),
                        KeyCode::Key8 | KeyCode::Numpad8 => self.game_ui.handle_quantity_input('8'),
                        KeyCode::Key9 | KeyCode::Numpad9 => self.game_ui.handle_quantity_input('9'),
                        _ => {}
                    }
                }
            }
            None => {}
        }
        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) -> GameResult {
        self.game_ui.update_mouse_pos(x, y);
        Ok(())
    }
}

fn main() -> GameResult {
    // Create window configuration
    let window_setup = WindowSetup::default()
        .title("8-Bit RuneScape")
        .vsync(true);

    let window_mode = WindowMode::default()
        .dimensions(1024.0, 768.0)
        .resizable(false);

    // Create context and window
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("8bitrs", "Alexander Mack")
        .window_setup(window_setup)
        .window_mode(window_mode)
        .build()?;

    // Create and run game
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
} 