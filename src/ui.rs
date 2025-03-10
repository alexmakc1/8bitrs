use ggez::{graphics::{self, Canvas, Color}, GameResult};
use ggez::glam::Vec2;
use crate::skills::Skills;
use crate::inventory::{Inventory, DroppedItem};
use crate::equipment::Equipment;
use crate::inventory::ArmorSlot;
use crate::entity::Entity;
use crate::world::{Tree, FishingSpot};
use crate::sprites::SpriteManager;

#[derive(Debug)]
pub struct ContextMenuItem {
    pub text: String,
    pub action: ContextMenuAction,
}

#[derive(Clone, Debug)]
pub enum ContextMenuAction {
    ChopTree,
    PickupItem,
    Attack,
    Fish,
    Examine(String),
    None,
}

pub struct ContextMenu {
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    items: Vec<ContextMenuItem>,
}

impl ContextMenu {
    pub fn new() -> Self {
        ContextMenu {
            visible: false,
            x: 0.0,
            y: 0.0,
            items: Vec::new(),
        }
    }

    pub fn show(&mut self, x: f32, y: f32, actions: Vec<(String, ContextMenuAction)>) {
        self.visible = true;
        self.x = x;
        self.y = y;
        self.items = actions.into_iter()
            .map(|(text, action)| ContextMenuItem { text, action })
            .collect();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
    }

    pub fn draw(&self, canvas: &mut Canvas) -> GameResult {
        if !self.visible {
            return Ok(());
        }

        let item_height = 20.0;
        let menu_width = 100.0;
        let menu_height = self.items.len() as f32 * item_height;

        // Draw menu background
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(self.x, self.y))
                .scale(Vec2::new(menu_width, menu_height))
                .color(Color::new(0.0, 0.0, 0.0, 0.8)),
        );

        // Draw menu items
        for (i, item) in self.items.iter().enumerate() {
            let text = graphics::Text::new(item.text.clone());
            canvas.draw(
                &text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.x + 5.0, self.y + i as f32 * item_height))
                    .color(Color::WHITE),
            );
        }

        Ok(())
    }

    pub fn handle_click(&self, x: f32, y: f32) -> Option<ContextMenuAction> {
        if !self.visible {
            return None;
        }

        let item_height = 20.0;
        
        // Check if click is within menu bounds
        if x < self.x || x > self.x + 100.0 || y < self.y || y > self.y + (self.items.len() as f32 * item_height) {
            return None;
        }

        // Calculate which item was clicked
        let item_index = ((y - self.y) / item_height) as usize;
        if item_index < self.items.len() {
            Some(self.items[item_index].action.clone())
        } else {
            None
        }
    }
}

pub struct GameUI {
    pub inventory_visible: bool,
    pub context_menu: ContextMenu,
    selected_slot: Option<usize>,
    tooltip_text: Option<String>,
    mouse_x: f32,
    mouse_y: f32,
    skills_menu_visible: bool,
    sprite_manager: &'static SpriteManager,
}

impl GameUI {
    pub fn new(sprite_manager: &'static SpriteManager) -> Self {
        Self {
            inventory_visible: false,
            context_menu: ContextMenu::new(),
            selected_slot: None,
            tooltip_text: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            skills_menu_visible: false,
            sprite_manager,
        }
    }

    pub fn toggle_inventory(&mut self) {
        self.inventory_visible = !self.inventory_visible;
    }

    pub fn toggle_skills_menu(&mut self) {
        self.skills_menu_visible = !self.skills_menu_visible;
    }

    pub fn is_menu_visible(&self) -> bool {
        self.inventory_visible || self.skills_menu_visible || self.context_menu.visible
    }

    pub fn get_selected_slot(&self) -> Option<usize> {
        self.selected_slot
    }

    pub fn select_slot(&mut self, slot: usize) {
        self.selected_slot = Some(slot);
    }

    pub fn clear_selection(&mut self) {
        self.selected_slot = None;
    }

    pub fn set_tooltip(&mut self, text: Option<String>) {
        self.tooltip_text = text;
    }

    pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn draw(&mut self, canvas: &mut Canvas, skills: &Skills, inventory: &Inventory, equipment: &Equipment, dropped_items: &[DroppedItem], player_x: f32, player_y: f32, entities: &[Entity], trees: &[Tree], fishing_spots: &[FishingSpot]) -> GameResult {
        // Clear tooltip at start of draw
        self.tooltip_text = None;

        // Draw inventory if visible
        if self.inventory_visible {
            // Draw inventory background
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(50.0, 80.0))
                    .scale(Vec2::new(220.0, 340.0))
                    .color(Color::new(0.0, 0.0, 0.0, 0.8)),
            );

            // Draw equipment section
            let equip_text = graphics::Text::new("Equipment:".to_string());
            canvas.draw(
                &equip_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(70.0, 85.0))
                    .color(Color::WHITE),
            );

            // Draw equipped items
            let equipped_items = [
                ("Weapon", equipment.get_weapon()),
                ("Head", equipment.get_armor(&ArmorSlot::Head)),
                ("Body", equipment.get_armor(&ArmorSlot::Body)),
                ("Legs", equipment.get_armor(&ArmorSlot::Legs)),
            ];

            for (i, (slot_name, item)) in equipped_items.iter().enumerate() {
                let x = 70.0 + i as f32 * 45.0;
                let y = 105.0;

                // Draw slot background
                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(x, y))
                        .scale(Vec2::new(40.0, 40.0))
                        .color(Color::new(0.4, 0.4, 0.4, 0.8)),
                );

                // Draw slot label
                let label = graphics::Text::new(slot_name.chars().next().unwrap_or('?').to_string());
                canvas.draw(
                    &label,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(x + 2.0, y + 2.0))
                        .color(Color::new(0.7, 0.7, 0.7, 0.5)),
                );

                // Draw equipped item if any
                if let Some(item) = item {
                    // Check if mouse is hovering over this slot
                    if self.mouse_x >= x && self.mouse_x <= x + 40.0 && 
                       self.mouse_y >= y && self.mouse_y <= y + 40.0 {
                        self.tooltip_text = Some(format!("Equipped: {}", item.name));
                    }

                    // Convert item name to sprite name
                    let sprite_name = item.name.to_lowercase().replace(" ", "_");
                    
                    // Draw item sprite
                    if let Some(sprite) = self.sprite_manager.get_sprite(&sprite_name) {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 4.0, y + 4.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );
                    } else {
                        println!("Missing sprite for item: {}", sprite_name);
                        // Fallback to letter if sprite not found
                        let text = graphics::Text::new(item.name.chars().next().unwrap_or('?').to_string());
                        canvas.draw(
                            &text,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 15.0, y + 15.0))
                                .color(Color::WHITE),
                        );
                    }
                }
            }

            // Draw inventory section
            let inv_text = graphics::Text::new("Inventory:".to_string());
            canvas.draw(
                &inv_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(70.0, 155.0))
                    .color(Color::WHITE),
            );

            // Draw inventory slots
            for i in 0..28 {
                let row = i / 4;
                let col = i % 4;
                let x = 70.0 + col as f32 * 45.0;
                let y = 175.0 + row as f32 * 45.0;

                // Draw slot background
                let slot_color = if Some(i) == self.selected_slot {
                    Color::new(0.5, 0.5, 0.5, 0.8)
                } else {
                    Color::new(0.3, 0.3, 0.3, 0.8)
                };

                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(x, y))
                        .scale(Vec2::new(40.0, 40.0))
                        .color(slot_color),
                );

                // Draw item in slot if it exists
                if let Some(item) = inventory.get_items().get(i).and_then(|opt| opt.as_ref()) {
                    // Check if mouse is hovering over this slot
                    if self.mouse_x >= x && self.mouse_x <= x + 40.0 && 
                       self.mouse_y >= y && self.mouse_y <= y + 40.0 {
                        self.tooltip_text = Some(item.name.clone());
                    }

                    // Convert item name to sprite name
                    let sprite_name = item.name.to_lowercase().replace(" ", "_");
                    
                    // Draw item sprite
                    if let Some(sprite) = self.sprite_manager.get_sprite(&sprite_name) {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 4.0, y + 4.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );
                    } else {
                        println!("Missing sprite for item: {}", sprite_name);
                        // Fallback to letter if sprite not found
                        let text = graphics::Text::new(item.name.chars().next().unwrap_or('?').to_string());
                        canvas.draw(
                            &text,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 15.0, y + 15.0))
                                .color(Color::WHITE),
                        );
                    }
                }
            }
        }

        // Draw skills menu if visible
        if self.skills_menu_visible {
            // Draw skills background
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(300.0, 80.0))
                    .scale(Vec2::new(250.0, 300.0))
                    .color(Color::new(0.0, 0.0, 0.0, 0.8)),
            );

            // Draw skill levels and XP
            let mut y = 100.0;
            let skills_text = [
                ("Attack", &skills.attack),
                ("Strength", &skills.strength),
                ("Defense", &skills.defense),
                ("Woodcutting", &skills.woodcutting),
                ("Fishing", &skills.fishing),
                ("Cooking", &skills.cooking),
                ("Firemaking", &skills.firemaking),
            ];

            for (skill_name, skill) in skills_text.iter() {
                let text = graphics::Text::new(format!("{}: {} (XP: {})", skill_name, skill.get_level(), skill.get_experience()));
                canvas.draw(
                    &text,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(320.0, y))
                        .color(Color::WHITE),
                );
                y += 30.0;
            }
        }

        // Draw context menu if visible
        self.context_menu.draw(canvas)?;

        // Draw tooltip if there is one
        if let Some(text) = &self.tooltip_text {
            let tooltip_text = graphics::Text::new(text.clone());
            canvas.draw(
                &tooltip_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.mouse_x + 10.0, self.mouse_y + 10.0))
                    .color(Color::WHITE),
            );
        }

        Ok(())
    }
} 