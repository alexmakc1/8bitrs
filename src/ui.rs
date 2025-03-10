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
    menu_bar_height: f32,
    messages: Vec<String>,
    max_messages: usize,
    message_scroll: f32,
    message_window_height: f32,
}

impl GameUI {
    pub fn new(sprite_manager: &'static SpriteManager) -> Self {
        Self {
            inventory_visible: true,
            context_menu: ContextMenu::new(),
            selected_slot: None,
            tooltip_text: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            skills_menu_visible: false,
            sprite_manager,
            menu_bar_height: 40.0,
            messages: Vec::new(),
            max_messages: 50,
            message_scroll: 0.0,
            message_window_height: 150.0,
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        self.message_scroll = 0.0;
    }

    fn wrap_text(&self, text: &str, max_width: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let font_width = 8.0; // Approximate width of each character in pixels

        for word in text.split_whitespace() {
            let word_width = word.len() as f32 * font_width;
            let space_width = if current_line.is_empty() { 0.0 } else { font_width };

            if current_width + word_width + space_width > max_width {
                if !current_line.is_empty() {
                    lines.push(current_line.trim().to_string());
                }
                current_line = word.to_string();
                current_width = word_width;
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
                current_width += word_width + space_width;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
        }

        lines
    }

    pub fn toggle_skills_menu(&mut self) {
        self.skills_menu_visible = !self.skills_menu_visible;
        if self.skills_menu_visible {
            self.inventory_visible = false;
        }
    }

    pub fn toggle_inventory(&mut self) {
        self.inventory_visible = !self.inventory_visible;
        if self.inventory_visible {
            self.skills_menu_visible = false;
        }
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
        self.tooltip_text = None;

        let screen_height = 768.0; // Window height
        let menu_y = screen_height - self.menu_bar_height;
        let message_y = menu_y - self.message_window_height;

        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(0.0, message_y))
                .scale(Vec2::new(1024.0, self.message_window_height))
                .color(Color::new(0.0, 0.0, 0.0, 0.8)),
        );

        let mut y = message_y + 10.0;
        let max_width = 1000.0;
        let line_height = 20.0;

        for message in self.messages.iter().rev() {
            let wrapped_lines = self.wrap_text(message, max_width);
            
            for line in wrapped_lines.iter().rev() {
                let line_text = graphics::Text::new(line.clone());
                canvas.draw(
                    &line_text,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(10.0, y + self.message_scroll))
                        .color(Color::WHITE),
                );
                y += line_height;
            }
        }

        let scroll_bar_height = 10.0;
        let scroll_bar_y = menu_y - scroll_bar_height;
        let scroll_percent = (self.message_scroll / (self.message_window_height - 20.0)).clamp(0.0, 1.0);
        let scroll_bar_x = scroll_percent * (1024.0 - 20.0);
        
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(scroll_bar_x, scroll_bar_y))
                .scale(Vec2::new(20.0, scroll_bar_height))
                .color(Color::WHITE),
        );

        if self.inventory_visible || self.skills_menu_visible {
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(10.0, 10.0))
                    .scale(Vec2::new(220.0, 340.0))
                    .color(Color::new(0.0, 0.0, 0.0, 0.8)),
            );
        }

        if self.inventory_visible {
            let equip_text = graphics::Text::new("Equipment:".to_string());
            canvas.draw(
                &equip_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(30.0, 125.0))
                    .color(Color::WHITE),
            );

            let equipped_items = [
                ("Weapon", equipment.get_weapon()),
                ("Head", equipment.get_armor(&ArmorSlot::Head)),
                ("Body", equipment.get_armor(&ArmorSlot::Body)),
                ("Legs", equipment.get_armor(&ArmorSlot::Legs)),
            ];

            for (i, (slot_name, item)) in equipped_items.iter().enumerate() {
                let x = 30.0 + i as f32 * 45.0;
                let y = 145.0;

                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(x, y))
                        .scale(Vec2::new(40.0, 40.0))
                        .color(Color::new(0.4, 0.4, 0.4, 0.8)),
                );

                let label = graphics::Text::new(slot_name.chars().next().unwrap_or('?').to_string());
                canvas.draw(
                    &label,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(x + 2.0, y + 2.0))
                        .color(Color::new(0.7, 0.7, 0.7, 0.5)),
                );

                if let Some(item) = item {
                    if self.mouse_x >= x && self.mouse_x <= x + 40.0 && 
                       self.mouse_y >= y && self.mouse_y <= y + 40.0 {
                        self.tooltip_text = Some(format!("Equipped: {}", item.name));
                    }

                    let sprite_name = item.name.to_lowercase().replace(" ", "_");
                    
                    if let Some(sprite) = self.sprite_manager.get_sprite(&sprite_name) {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 4.0, y + 4.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );
                    } else {
                        println!("Missing sprite for item: {}", sprite_name);
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

            let inv_text = graphics::Text::new("Inventory:".to_string());
            canvas.draw(
                &inv_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(30.0, 195.0))
                    .color(Color::WHITE),
            );

            for i in 0..28 {
                let row = i / 4;
                let col = i % 4;
                let x = 30.0 + col as f32 * 45.0;
                let y = 215.0 + row as f32 * 45.0;

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

                if let Some(item) = inventory.get_items().get(i).and_then(|opt| opt.as_ref()) {
                    if self.mouse_x >= x && self.mouse_x <= x + 40.0 && 
                       self.mouse_y >= y && self.mouse_y <= y + 40.0 {
                        self.tooltip_text = Some(item.name.clone());
                    }

                    let sprite_name = item.name.to_lowercase().replace(" ", "_");
                    
                    if let Some(sprite) = self.sprite_manager.get_sprite(&sprite_name) {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 4.0, y + 4.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );
                    } else {
                        println!("Missing sprite for item: {}", sprite_name);
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

        if self.skills_menu_visible {
            let skills_text = graphics::Text::new("Skills:".to_string());
            canvas.draw(
                &skills_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(30.0, 125.0))
                    .color(Color::WHITE),
            );

            let mut y = 145.0;
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
                        .dest(Vec2::new(30.0, y))
                        .color(Color::WHITE),
                );
                y += 30.0;
            }
        }

        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(0.0, menu_y))
                .scale(Vec2::new(1024.0, self.menu_bar_height))
                .color(Color::new(0.0, 0.0, 0.0, 0.8)),
        );

        let button_width = 100.0;
        let button_height = 30.0;
        let button_spacing = 10.0;
        let start_x = 10.0;

        let inventory_button_color = if self.inventory_visible {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            Color::new(0.3, 0.3, 0.3, 1.0)
        };
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(start_x, menu_y + 5.0))
                .scale(Vec2::new(button_width, button_height))
                .color(inventory_button_color),
        );
        let inventory_text = graphics::Text::new("Inventory");
        canvas.draw(
            &inventory_text,
            graphics::DrawParam::new()
                .dest(Vec2::new(start_x + 10.0, menu_y + 10.0))
                .color(Color::WHITE),
        );

        let stats_button_color = if self.skills_menu_visible {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            Color::new(0.3, 0.3, 0.3, 1.0)
        };
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(start_x + button_width + button_spacing, menu_y + 5.0))
                .scale(Vec2::new(button_width, button_height))
                .color(stats_button_color),
        );
        let stats_text = graphics::Text::new("Stats");
        canvas.draw(
            &stats_text,
            graphics::DrawParam::new()
                .dest(Vec2::new(start_x + button_width + button_spacing + 10.0, menu_y + 10.0))
                .color(Color::WHITE),
        );

        self.context_menu.draw(canvas)?;

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

    pub fn handle_menu_click(&mut self, x: f32, y: f32) -> bool {
        let screen_height = 768.0; // Window height
        let menu_y = screen_height - self.menu_bar_height;
        let message_y = menu_y - self.message_window_height;

        if y >= message_y && y < menu_y {
            if y > menu_y - 20.0 {
                let scroll_percent = (x / 1024.0).clamp(0.0, 1.0);
                self.message_scroll = scroll_percent * (self.message_window_height - 20.0);
            }
            return true;
        }

        if y >= menu_y && y <= screen_height {
            let button_width = 100.0;
            let button_height = 30.0;
            let button_spacing = 10.0;
            let start_x = 10.0;

            if x >= start_x && x <= start_x + button_width && 
               y >= menu_y + 5.0 && y <= menu_y + 5.0 + button_height {
                self.toggle_inventory();
                return true;
            }

            if x >= start_x + button_width + button_spacing && 
               x <= start_x + button_width + button_spacing + button_width && 
               y >= menu_y + 5.0 && y <= menu_y + 5.0 + button_height {
                self.toggle_skills_menu();
                return true;
            }
        }
        false
    }
} 