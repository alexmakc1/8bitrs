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

use skills::Skills;
use ui::{GameUI, ContextMenuAction};
use combat::Combat;
use entity::Entity;
use inventory::{Inventory, Item, DroppedItem, ItemType, ToolType, ResourceType};
use equipment::Equipment;
use world::{Tree, Fire, FishingSpot, FishType};
use save::{SaveData, create_save_data};
use sprites::SpriteManager;
use world_objects::{WorldObject, ObjectType};

#[derive(Clone)]
enum PendingAction {
    ChopTree(f32, f32),
    PickupItem(usize),
    Attack,
    Fish(f32, f32),
    None,
}

#[derive(Clone)]
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
    sprite_manager: SpriteManager,
    world_objects: Vec<WorldObject>,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let sprite_manager = SpriteManager::new(ctx)?;
        
        let mut state = Self {
            player_x: 512.0,
            player_y: 384.0,
            camera_x: 0.0,
            camera_y: 0.0,
            movement_speed: 4.0,
            skills: Skills::new(),
            game_ui: GameUI::new(),
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
            sprite_manager,
            world_objects: Vec::new(),
        };

        // Add starting equipment
        state.inventory.add_item(Item::bronze_sword());
        state.inventory.add_item(Item::bronze_helmet());
        state.inventory.add_item(Item::bronze_platebody());
        state.inventory.add_item(Item::bronze_platelegs());
        state.inventory.add_item(Item::bronze_axe());
        state.inventory.add_item(Item::tinderbox());

        state.spawn_world_objects();
        Ok(state)
    }

    fn spawn_world_objects(&mut self) {
        let mut rng = rand::thread_rng();
        
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
                    let x = center_x + dx as f32 * 40.0 + rng.gen_range(-10.0..10.0);
                    let y = center_y + dy as f32 * 40.0 + rng.gen_range(-10.0..10.0);
                    
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
    }

    fn check_collision(&self, x: f32, y: f32) -> bool {
        const PLAYER_SIZE: f32 = 32.0;
        
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

            if distance > 5.0 {
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
                    PendingAction::ChopTree(x, y) => {
                        if let Some((index, _)) = self.trees.iter().enumerate()
                            .find(|(_, t)| t.is_near(x, y) && !t.is_chopped())
                        {
                            self.ongoing_action = OngoingAction::ChoppingTree { x, y, tree_index: index };
                            self.action_timer = 0.0; // Start chopping immediately
                        }
                    }
                    PendingAction::Attack => {
                        if let Some((index, _)) = self.entities.iter().enumerate()
                            .find(|(_, e)| e.is_near(self.player_x, self.player_y) && e.is_alive())
                        {
                            self.ongoing_action = OngoingAction::Fighting { target_index: index };
                            self.action_timer = 0.0; // Attack immediately
                        }
                    }
                    PendingAction::Fish(x, y) => {
                        if let Some((index, _)) = self.fishing_spots.iter().enumerate()
                            .find(|(_, s)| s.is_near(x, y))
                        {
                            self.ongoing_action = OngoingAction::Fishing { x, y, spot_index: index };
                            self.action_timer = 0.0; // Start fishing immediately
                        }
                    }
                    PendingAction::PickupItem(index) => {
                        if index < self.dropped_items.len() {
                            let dropped_item = &self.dropped_items[index];
                            if self.inventory.add_item(dropped_item.item.clone()) {
                                self.dropped_items.remove(index);
                                println!("You pick up the item.");
                            } else {
                                println!("Your inventory is full.");
                            }
                        }
                    }
                    PendingAction::None => {}
                }
                self.pending_action = PendingAction::None;
            }
        }
    }

    fn save_game(&self) {
        let save_data = create_save_data(
            self.player_x,
            self.player_y,
            &self.skills,
            &self.player_combat,
            &self.inventory,
            &self.equipment,
        );

        if let Err(e) = save_data.save_to_file("save_game.json") {
            println!("Error saving game: {}", e);
        }
    }

    fn set_destination(&mut self, x: f32, y: f32, action: PendingAction) {
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
                OngoingAction::ChoppingTree { x, y, tree_index } => {
                    if *tree_index < self.trees.len() {
                        let tree = &mut self.trees[*tree_index];
                        if tree.is_chopped() {
                            self.cancel_ongoing_action();
                            return;
                        }

                        // Check if we're still in range
                        let dx = *x - self.player_x;
                        let dy = *y - self.player_y;
                        if (dx * dx + dy * dy).sqrt() > 40.0 {
                            self.set_destination(*x, *y, PendingAction::ChopTree(*x, *y));
                            return;
                        }

                        let axe = self.inventory.get_items().iter()
                            .filter_map(|item| item.as_ref())
                            .find(|item| matches!(&item.item_type, ItemType::Tool(ToolType::Axe { .. })));

                        if tree.try_chop(&self.skills, axe) {
                            if self.inventory.add_item(Item::logs()) {
                                println!("You get some logs.");
                                self.skills.gain_woodcutting_xp(25);
                                self.action_timer = 3.0; // Set timer for next chop
                            } else {
                                println!("Your inventory is full.");
                                self.cancel_ongoing_action();
                            }
                        } else {
                            self.action_timer = 3.0; // Try again in 3 seconds
                        }
                    } else {
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

                        // Check if we're still in range
                        if !target.is_near(self.player_x, self.player_y) {
                            let target_pos = target.get_position();
                            self.set_destination(target_pos.0, target_pos.1, PendingAction::Attack);
                            return;
                        }

                        self.attack_nearest_goblin();
                        self.action_timer = 2.4; // Set timer for next attack (typical RuneScape attack speed)
                    } else {
                        self.cancel_ongoing_action();
                    }
                }
                OngoingAction::Fishing { x, y, spot_index } => {
                    if *spot_index < self.fishing_spots.len() {
                        let spot = &self.fishing_spots[*spot_index];
                        
                        // Check if we're still in range
                        let dx = *x - self.player_x;
                        let dy = *y - self.player_y;
                        if (dx * dx + dy * dy).sqrt() > 40.0 {
                            self.set_destination(*x, *y, PendingAction::Fish(*x, *y));
                            return;
                        }

                        // Find fishing rod and check for bait
                        let rod = self.inventory.get_items().iter()
                            .filter_map(|item| item.as_ref())
                            .find(|item| matches!(&item.item_type, ItemType::Tool(ToolType::FishingRod { .. })));
                        
                        let has_bait = self.inventory.get_items().iter()
                            .filter_map(|item| item.as_ref())
                            .any(|item| matches!(&item.item_type, ItemType::Resource(ResourceType::Bait)));

                        if let Some(fish) = spot.try_fish(&self.skills, rod, has_bait) {
                            // Remove bait if needed
                            if matches!(spot.fish_type, FishType::Trout) {
                                // Find and remove one bait
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
                                println!("You catch a {}.", fish.name);
                                self.skills.gain_fishing_xp(match spot.fish_type {
                                    FishType::Shrimp => 10,
                                    FishType::Trout => 50,
                                });
                                self.action_timer = 3.0; // Set timer for next fishing attempt
                            } else {
                                println!("Your inventory is full.");
                                self.cancel_ongoing_action();
                            }
                        } else {
                            println!("You fail to catch anything.");
                            self.action_timer = 3.0; // Try again in 3 seconds
                        }
                    } else {
                        self.cancel_ongoing_action();
                    }
                }
                OngoingAction::None => {}
            }
        }
    }

    fn attack_nearest_goblin(&mut self) {
        // Find the nearest goblin
        if let Some(goblin_index) = self.entities.iter()
            .enumerate()
            .filter(|(_, e)| e.is_near(self.player_x, self.player_y))
            .find(|(_, e)| e.is_alive())
            .map(|(i, _)| i)
        {
            let goblin = &mut self.entities[goblin_index];
            if let Some(goblin_combat) = goblin.get_combat_mut() {
                // Get equipment bonuses
                let attack_bonus = self.equipment.get_total_attack_bonus();
                let strength_bonus = self.equipment.get_total_strength_bonus();
                let defense_bonus = self.equipment.get_total_defense_bonus();

                // Player attacks goblin
                if let Some(damage) = self.player_combat.attack(&self.skills, &Skills::new(), attack_bonus, strength_bonus, 0) {
                    println!("Player hits goblin for {} damage! (Attack: {}, Strength: {}, Defense: {})", 
                        damage, attack_bonus, strength_bonus, defense_bonus);
                    goblin_combat.take_damage(damage as i32);
                    self.skills.gain_attack_xp(4); // Gain some attack XP
                    
                    // If goblin dies
                    if goblin_combat.is_dead() {
                        println!("Goblin defeated!");
                        // Get drops and add them to the ground
                        if let Some(drops) = goblin.interact(&mut self.skills) {
                            for item in drops {
                                self.dropped_items.push(DroppedItem::new(
                                    item,
                                    goblin.get_position().0,
                                    goblin.get_position().1,
                                ));
                            }
                        }
                        // Gain combat XP for kill
                        self.skills.gain_attack_xp(10);
                        self.skills.gain_strength_xp(10);
                        self.skills.gain_defense_xp(10);
                    } else {
                        // Goblin counterattacks
                        if let Some(damage) = goblin_combat.attack(&Skills::new(), &self.skills, 0, 0, defense_bonus) {
                            println!("Goblin hits player for {} damage!", damage);
                            self.player_combat.take_damage(damage as i32);
                            self.skills.gain_defense_xp(4); // Gain some defense XP
                        } else {
                            println!("Goblin misses!");
                        }
                    }
                } else {
                    println!("Player misses!");
                }
            }
        }
    }

    fn handle_inventory_click(&mut self, slot: usize, button: MouseButton) {
        println!("Handling inventory click for slot {} with button {:?}", slot, button);
        if let Some(item) = self.inventory.get_item(slot) {
            println!("Found item: {}", item.name);
            match button {
                MouseButton::Left => {
                    if let Some(selected_slot) = self.selected_item {
                        println!("Have selected slot: {}", selected_slot);
                        // Handle item-on-item interaction
                        if let Some(selected_item) = self.inventory.get_item(selected_slot) {
                            println!("Attempting to use {} on {}", selected_item.name, item.name);
                            match (&selected_item.item_type, &item.item_type) {
                                (ItemType::Tool(ToolType::Tinderbox), ItemType::Resource(ResourceType::Logs { firemaking_level })) |
                                (ItemType::Resource(ResourceType::Logs { firemaking_level }), ItemType::Tool(ToolType::Tinderbox)) => {
                                    if u32::from(self.skills.firemaking.get_level()) >= *firemaking_level {
                                        // Create fire at player's position
                                        self.fires.push(Fire::new(self.player_x, self.player_y));
                                        // Remove logs (always remove from non-tinderbox slot)
                                        let logs_slot = if matches!(selected_item.item_type, ItemType::Tool(ToolType::Tinderbox)) {
                                            slot
                                        } else {
                                            selected_slot
                                        };
                                        if self.inventory.remove_item(logs_slot).is_some() {
                                            // Grant firemaking XP
                                            self.skills.gain_firemaking_xp(40);
                                            println!("You light a fire.");
                                        }
                                    } else {
                                        println!("You need level {} Firemaking to light these logs.", firemaking_level);
                                    }
                                }
                                _ => {
                                    // Check if trying to cook fish on fire
                                    if let ItemType::Resource(ResourceType::RawFish { cooking_level, .. }) = &item.item_type {
                                        // Find nearby fire
                                        if let Some(fire) = self.fires.iter()
                                            .find(|f| f.is_near(self.player_x, self.player_y))
                                        {
                                            if u32::from(self.skills.cooking.get_level()) >= *cooking_level {
                                                // Clone the item before we start modifying the inventory
                                                let raw_fish = item.clone();
                                                
                                                // Try to cook the fish
                                                if let Some(cooked_item) = fire.try_cook(&raw_fish, self.skills.cooking.get_level()) {
                                                    // Remove raw fish
                                                    self.inventory.remove_item(slot);
                                                    // Add cooked/burnt fish
                                                    if self.inventory.add_item(cooked_item.clone()) {
                                                        match cooked_item.name.as_str() {
                                                            "Burnt Fish" => println!("You accidentally burn the fish."),
                                                            _ => {
                                                                println!("You successfully cook the {}.", raw_fish.name.strip_prefix("Raw ").unwrap_or(&raw_fish.name));
                                                                self.skills.gain_cooking_xp(30);
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                println!("You need level {} Cooking to cook this fish.", cooking_level);
                                            }
                                        } else {
                                            println!("You need to be near a fire to cook food.");
                                        }
                                    }
                                }
                            }
                        }
                        self.selected_item = None;
                        self.game_ui.clear_selection();
                    } else {
                        // Select item for use
                        self.selected_item = Some(slot);
                        self.game_ui.select_slot(slot);
                        println!("Selected slot {} with item {}", slot, item.name);
                        match &item.item_type {
                            ItemType::Tool(ToolType::Tinderbox) => {
                                println!("Use the tinderbox on logs to light them.");
                            }
                            ItemType::Resource(ResourceType::Logs { .. }) => {
                                println!("Use a tinderbox on the logs to light them.");
                            }
                            ItemType::Weapon(_) | ItemType::Armor(_) => {
                                // Handle equipment
                                if let Some(item) = self.inventory.remove_item(slot) {
                                    let old_item = match &item.item_type {
                                        ItemType::Weapon(_) => self.equipment.equip_weapon(item),
                                        ItemType::Armor(_) => self.equipment.equip_armor(item),
                                        _ => None,
                                    };
                                    
                                    if let Some(old_item) = old_item {
                                        self.inventory.add_item(old_item);
                                    }
                                }
                                self.selected_item = None;
                                self.game_ui.clear_selection();
                            }
                            ItemType::Food(_) => {
                                // Use item (like eating food)
                                self.inventory.use_item(slot, &mut self.player_combat);
                                self.selected_item = None;
                                self.game_ui.clear_selection();
                            }
                            _ => {}
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

    fn try_pickup_item(&mut self) {
        let pickup_range = 40.0;
        self.dropped_items.retain_mut(|item| {
            let dx = item.x - self.player_x;
            let dy = item.y - self.player_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < pickup_range && item.can_pickup() {
                if self.inventory.add_item(item.item.clone()) {
                    println!("You picked up a {}.", item.item.name);
                    return false; // Remove item from ground
                }
            }
            true // Keep item on ground
        });
    }

    fn try_chop_tree(&mut self) {
        // Find the nearest tree
        if let Some(tree) = self.trees.iter_mut()
            .filter(|t| t.is_near(self.player_x, self.player_y))
            .find(|t| !t.is_chopped())
        {
            // Find axe in inventory
            let axe = self.inventory.get_items().iter()
                .filter_map(|item| item.as_ref())
                .find(|item| matches!(&item.item_type, ItemType::Tool(ToolType::Axe { .. })));

            if tree.try_chop(&self.skills, axe) {
                // Add logs to inventory
                if self.inventory.add_item(Item::logs()) {
                    println!("You get some logs.");
                    self.skills.gain_woodcutting_xp(25);
                } else {
                    println!("Your inventory is full.");
                }
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
                            actions.push(("Chop tree".to_string(), ContextMenuAction::ChopTree));
                            actions.push(("Examine tree".to_string(), ContextMenuAction::Examine("A sturdy tree good for woodcutting.".to_string())));
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
                    }
                    break; // Only show options for the first object found
                }
            }

            // Check for nearby dropped items
            if let Some((index, item)) = self.dropped_items.iter().enumerate()
                .find(|(_, i)| {
                    let dx = i.x - world_x;
                    let dy = i.y - world_y;
                    (dx * dx + dy * dy).sqrt() < 40.0
                })
            {
                actions.push(("Pick up".to_string(), ContextMenuAction::PickupItem));
                actions.push(("Examine".to_string(), ContextMenuAction::Examine(format!("It's a {}.", item.item.name))));
            }

            // Check for nearby goblins
            if let Some(_goblin) = self.entities.iter()
                .find(|e| e.is_near(world_x, world_y) && e.is_alive())
            {
                actions.push(("Attack".to_string(), ContextMenuAction::Attack));
                actions.push(("Examine".to_string(), ContextMenuAction::Examine("A mean-looking goblin.".to_string())));
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
                self.set_destination(x, y, PendingAction::ChopTree(x, y));
            }
            ContextMenuAction::PickupItem => {
                if let Some((index, _)) = self.dropped_items.iter().enumerate()
                    .find(|(_, i)| {
                        let dx = i.x - x;
                        let dy = i.y - y;
                        (dx * dx + dy * dy).sqrt() < 40.0
                    })
                {
                    self.set_destination(x, y, PendingAction::PickupItem(index));
                }
            }
            ContextMenuAction::Attack => {
                self.set_destination(x, y, PendingAction::Attack);
            }
            ContextMenuAction::Fish => {
                self.set_destination(x, y, PendingAction::Fish(x, y));
            }
            ContextMenuAction::Examine(text) => {
                println!("{}", text);
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

        // Draw UI elements (these don't use camera offset)
        self.game_ui.draw(
            &mut canvas,
            &self.skills,
            &self.inventory,
            &self.equipment,
            &self.dropped_items,
            self.player_x,
            self.player_y,
            &self.entities,
            &self.trees,
            &self.fishing_spots,
        )?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> GameResult {
        // Check for minimap clicks first
        if let Some((world_x, world_y)) = self.game_ui.minimap.handle_click(x, y, self.player_x, self.player_y) {
            self.set_destination(world_x, world_y, PendingAction::None);
            return Ok(());
        }

        // First check if we're clicking in the UI area
        if y >= 0.0 && y <= 50.0 {
            // UI bar click handling
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

        if self.game_ui.inventory_visible {
            // Check if click is in inventory area (adjusted coordinates)
            if x >= 70.0 && x <= 250.0 && y >= 100.0 && y <= 400.0 {
                let slot_x = ((x - 70.0) / 45.0).floor() as usize;
                let slot_y = ((y - 100.0) / 45.0).floor() as usize;
                let slot = slot_y * 4 + slot_x;
                
                if slot < self.inventory.get_items().len() {
                    self.handle_inventory_click(slot, button);
                }
            }
        } else if !self.game_ui.is_menu_visible() {
            // Only handle world clicks if no menu is visible
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

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::Tab) => {
                self.game_ui.toggle_skills_menu();
            }
            Some(KeyCode::I) => {
                self.game_ui.toggle_inventory();
            }
            Some(KeyCode::Space) => {
                if !self.game_ui.is_menu_visible() {
                    self.attack_nearest_goblin();
                }
            }
            Some(KeyCode::F) => {
                if !self.game_ui.is_menu_visible() {
                    self.try_pickup_item();
                }
            }
            Some(KeyCode::X) => {
                if !self.game_ui.is_menu_visible() {
                    self.try_chop_tree();
                }
            }
            Some(KeyCode::S) => {
                if input.mods.contains(ggez::input::keyboard::KeyMods::CTRL) {
                    self.save_game();
                }
            }
            Some(KeyCode::M) => {
                self.game_ui.minimap.visible = !self.game_ui.minimap.visible;
            }
            _ => {}
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