use ggez::{Context, GameResult};
use ggez::graphics::{self, Canvas, Color, Rect};
use ggez::glam::Vec2;
use crate::skills::Skills;
use crate::inventory::{Inventory, DroppedItem, ItemType};
use crate::equipment::Equipment;
use crate::inventory::ArmorSlot;
use crate::entity::Entity;
use crate::world::{Tree, FishingSpot};
use crate::sprites::SpriteManager;
use ggez::input::mouse::MouseButton;
use crate::bank::Bank;

#[derive(Debug)]
pub struct ContextMenuItem {
    pub text: String,
    pub action: ContextMenuAction,
}

#[derive(Debug, Clone)]
pub enum ContextMenuAction {
    ChopTree,
    PickupItem,
    Attack,
    Fish,
    OpenBank,
    Examine(String),
    WithdrawOne,
    WithdrawTen,
    WithdrawHundred,
    WithdrawAll,
    WithdrawX,
    DepositOne,
    DepositTen,
    DepositHundred,
    DepositX,
    DepositAll,
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
    pub context_menu: ContextMenu,
    pub inventory_visible: bool,
    pub skills_menu_visible: bool,
    pub equipment_screen_visible: bool,
    pub bank_visible: bool,
    mouse_x: f32,
    mouse_y: f32,
    tooltip_text: Option<String>,
    messages: Vec<String>,
    message_scroll: f32,
    message_window_height: f32,
    sprite_manager: &'static SpriteManager,
    selected_slot: Option<usize>,
    menu_bar_height: f32,
    max_messages: usize,
    pub selected_bank_slot: Option<usize>,
    pub selected_inventory_slot: Option<usize>,
    pub quantity_dialog_visible: bool,
    pub quantity_dialog_is_withdraw: bool,
    pub quantity_input: String,
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
            equipment_screen_visible: false,
            bank_visible: false,
            sprite_manager,
            menu_bar_height: 40.0,
            messages: Vec::new(),
            max_messages: 50,
            message_scroll: 0.0,
            message_window_height: 150.0,
            selected_bank_slot: None,
            selected_inventory_slot: None,
            quantity_dialog_visible: false,
            quantity_dialog_is_withdraw: true,
            quantity_input: String::new(),
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
            self.equipment_screen_visible = false;
        }
    }

    pub fn toggle_inventory(&mut self) {
        self.inventory_visible = !self.inventory_visible;
        if self.inventory_visible {
            self.skills_menu_visible = false;
            self.equipment_screen_visible = false;
        }
    }

    pub fn toggle_equipment_screen(&mut self) {
        self.equipment_screen_visible = !self.equipment_screen_visible;
        if self.equipment_screen_visible {
            self.inventory_visible = false;
            self.skills_menu_visible = false;
        }
    }

    pub fn toggle_bank(&mut self) {
        self.bank_visible = !self.bank_visible;
        if self.bank_visible {
            self.inventory_visible = true; // Always show inventory with bank
            self.skills_menu_visible = false;
            self.equipment_screen_visible = false;
        }
    }

    pub fn is_menu_visible(&self) -> bool {
        self.inventory_visible || self.skills_menu_visible || self.equipment_screen_visible || self.context_menu.visible
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

    pub fn draw(
        &mut self,
        canvas: &mut Canvas,
        skills: &Skills,
        inventory: &Inventory,
        equipment: &Equipment,
        bank: &Bank,
        player_x: f32,
        player_y: f32,
    ) -> GameResult {
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

        if self.inventory_visible || self.skills_menu_visible || self.equipment_screen_visible {
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(10.0, 10.0))
                    .scale(Vec2::new(220.0, 340.0))
                    .color(Color::new(0.0, 0.0, 0.0, 0.8)),
            );
        }

        if self.inventory_visible {
            let inv_text = graphics::Text::new("Inventory:".to_string());
            canvas.draw(
                &inv_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(30.0, 30.0))
                    .color(Color::WHITE),
            );

            for i in 0..28 {
                let row = i / 4;
                let col = i % 4;
                let x = 30.0 + col as f32 * 45.0;
                let y = 50.0 + row as f32 * 45.0;

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
                        self.tooltip_text = Some(format!("{} ({})", item.name, item.quantity));
                    }

                    let sprite_name = if item.name == "GP" {
                        "gp".to_string()
                    } else {
                        item.name.to_lowercase().replace(" ", "_")
                    };
                    
                    if let Some(sprite) = self.sprite_manager.get_sprite(&sprite_name) {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 4.0, y + 4.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );

                        // Always show quantity for stackable items or when quantity > 1
                        if item.is_stackable() || item.quantity > 1 {
                            let quantity_text = graphics::Text::new(item.quantity.to_string());
                            canvas.draw(
                                &quantity_text,
                                graphics::DrawParam::new()
                                    .dest(Vec2::new(x + 25.0, y + 2.0))
                                    .color(Color::WHITE),
                            );
                        }
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

        if self.equipment_screen_visible {
            // Draw equipment screen background
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(10.0, 10.0))
                    .scale(Vec2::new(220.0, 340.0))
                    .color(Color::new(0.0, 0.0, 0.0, 0.8)),
            );

            // Draw title
            let title = graphics::Text::new("Equipment");
            canvas.draw(
                &title,
                graphics::DrawParam::new()
                    .dest(Vec2::new(30.0, 20.0))
                    .color(Color::WHITE),
            );

            // Draw equipment slots
            let equipped_items = [
                ("Weapon", equipment.get_weapon()),
                ("Head", equipment.get_armor(&ArmorSlot::Head)),
                ("Body", equipment.get_armor(&ArmorSlot::Body)),
                ("Legs", equipment.get_armor(&ArmorSlot::Legs)),
            ];

            for (i, (slot_name, item)) in equipped_items.iter().enumerate() {
                let y = 60.0 + i as f32 * 45.0;
                
                // Draw slot background
                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(30.0, y))
                        .scale(Vec2::new(40.0, 40.0))
                        .color(Color::new(0.4, 0.4, 0.4, 0.8)),
                );

                // Draw slot name
                let slot_text = graphics::Text::new(slot_name.to_string());
                canvas.draw(
                    &slot_text,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(80.0, y + 10.0))
                        .color(Color::WHITE),
                );

                // Draw item if equipped
                if let Some(item) = item {
                    let sprite_name = item.name.to_lowercase().replace(" ", "_");
                    if let Some(sprite) = self.sprite_manager.get_sprite(&sprite_name) {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(34.0, y + 4.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );
                    }

                    // Show tooltip on hover
                    if self.mouse_x >= 30.0 && self.mouse_x <= 70.0 && 
                       self.mouse_y >= y && self.mouse_y <= y + 40.0 {
                        self.tooltip_text = Some(format!("{} (Click to unequip)", item.name));
                    }
                }
            }

            // Draw combat bonuses
            let y = 240.0;
            let bonuses = [
                ("Attack Bonus:", equipment.get_total_attack_bonus()),
                ("Strength Bonus:", equipment.get_total_strength_bonus()),
                ("Defense Bonus:", equipment.get_total_defense_bonus()),
            ];

            for (i, (label, value)) in bonuses.iter().enumerate() {
                let bonus_text = graphics::Text::new(format!("{} {}", label, value));
                canvas.draw(
                    &bonus_text,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(30.0, y + i as f32 * 25.0))
                        .color(Color::WHITE),
                );
            }
        }

        if self.bank_visible {
            // Draw bank window
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(250.0, 10.0))
                    .scale(Vec2::new(500.0, 600.0))
                    .color(Color::new(0.0, 0.0, 0.0, 0.8)),
            );

            // Draw close button background
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(720.0, 15.0))
                    .scale(Vec2::new(20.0, 20.0))
                    .color(Color::new(0.5, 0.0, 0.0, 0.8)),
            );

            // Draw close button (X)
            let close_button = graphics::Text::new("X");
            canvas.draw(
                &close_button,
                graphics::DrawParam::new()
                    .dest(Vec2::new(726.0, 17.0))
                    .color(Color::WHITE),
            );

            let bank_text = graphics::Text::new("Bank:".to_string());
            canvas.draw(
                &bank_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(270.0, 30.0))
                    .color(Color::WHITE),
            );

            // Draw instructions
            let instructions = graphics::Text::new("Left-click: Withdraw | Right-click: Deposit");
            canvas.draw(
                &instructions,
                graphics::DrawParam::new()
                    .dest(Vec2::new(270.0, 420.0))
                    .color(Color::WHITE),
            );

            // Draw bank slots (10x8 grid)
            for i in 0..80 {
                let row = i / 10;
                let col = i % 10;
                let x = 270.0 + col as f32 * 45.0;
                let y = 50.0 + row as f32 * 45.0;

                // Draw slot background
                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(Vec2::new(x, y))
                        .scale(Vec2::new(40.0, 40.0))
                        .color(Color::new(0.3, 0.3, 0.3, 0.8)),
                );

                if let Some(bank_slot) = bank.get_item(i) {
                    // Show tooltip on hover
                    if self.mouse_x >= x && self.mouse_x <= x + 40.0 && 
                       self.mouse_y >= y && self.mouse_y <= y + 40.0 {
                        self.tooltip_text = Some(format!("{} ({})", bank_slot.name, bank_slot.quantity));
                    }

                    let sprite_name = if bank_slot.name == "GP" {
                        "gp".to_string()
                    } else {
                        bank_slot.name.to_lowercase().replace(" ", "_")
                    };
                    
                    if let Some(sprite) = self.sprite_manager.get_sprite(&sprite_name) {
                        canvas.draw(
                            sprite,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 4.0, y + 4.0))
                                .scale(Vec2::new(2.0, 2.0))
                        );
                    } else {
                        println!("Missing sprite for item: {}", sprite_name);
                        let text = graphics::Text::new(bank_slot.name.chars().next().unwrap_or('?').to_string());
                        canvas.draw(
                            &text,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 15.0, y + 15.0))
                                .color(Color::WHITE),
                        );
                    }

                    // Always show quantity for stackable items or when quantity > 1
                    if bank_slot.is_stackable() || bank_slot.quantity > 1 {
                        let quantity_text = graphics::Text::new(bank_slot.quantity.to_string());
                        canvas.draw(
                            &quantity_text,
                            graphics::DrawParam::new()
                                .dest(Vec2::new(x + 25.0, y + 2.0))
                                .color(Color::WHITE),
                        );
                    }
                }
            }
        }

        // Draw menu bar
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(Vec2::new(0.0, menu_y))
                .scale(Vec2::new(1024.0, self.menu_bar_height))
                .color(Color::new(0.0, 0.0, 0.0, 0.8)),
        );

        // Draw menu buttons
        let buttons = [
            ("Inventory (I)", self.inventory_visible),
            ("Skills (K)", self.skills_menu_visible),
            ("Equipment (E)", self.equipment_screen_visible),
        ];

        for (i, (text, active)) in buttons.iter().enumerate() {
            let x = 10.0 + i as f32 * 120.0;
            let button_text = graphics::Text::new(text.to_string());
            
            // Draw button background
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(x, menu_y + 5.0))
                    .scale(Vec2::new(110.0, 30.0))
                    .color(if *active {
                        Color::new(0.4, 0.4, 0.4, 0.8)
                    } else {
                        Color::new(0.2, 0.2, 0.2, 0.8)
                    }),
            );

            // Draw button text
            canvas.draw(
                &button_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(x + 5.0, menu_y + 10.0))
                    .color(Color::WHITE),
            );
        }

        self.context_menu.draw(canvas)?;

        if let Some(text) = &self.tooltip_text {
            let tooltip = graphics::Text::new(text.clone());
            canvas.draw(
                &tooltip,
                graphics::DrawParam::new()
                    .dest(Vec2::new(self.mouse_x + 15.0, self.mouse_y - 15.0))
                    .color(Color::WHITE),
            );
        }

        // Draw quantity dialog if visible
        if self.quantity_dialog_visible {
            // Draw dialog background
            let dialog_width = 200.0;
            let dialog_height = 100.0;
            let dialog_x = 512.0 - dialog_width / 2.0; // Center horizontally
            let dialog_y = 384.0 - dialog_height / 2.0; // Center vertically

            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(dialog_x, dialog_y))
                    .scale(Vec2::new(dialog_width, dialog_height))
                    .color(Color::new(0.0, 0.0, 0.0, 0.9)),
            );

            // Draw dialog title
            let title = if self.quantity_dialog_is_withdraw {
                "Enter amount to withdraw:"
            } else {
                "Enter amount to deposit:"
            };
            let title_text = graphics::Text::new(title);
            canvas.draw(
                &title_text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(dialog_x + 10.0, dialog_y + 10.0))
                    .color(Color::WHITE),
            );

            // Draw input box background
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest(Vec2::new(dialog_x + 10.0, dialog_y + 40.0))
                    .scale(Vec2::new(180.0, 30.0))
                    .color(Color::new(0.2, 0.2, 0.2, 1.0)),
            );

            // Draw input text
            let input_text = if self.quantity_input.is_empty() {
                "Enter amount...".to_string()
            } else {
                self.quantity_input.clone()
            };
            let text = graphics::Text::new(input_text);
            canvas.draw(
                &text,
                graphics::DrawParam::new()
                    .dest(Vec2::new(dialog_x + 15.0, dialog_y + 45.0))
                    .color(if self.quantity_input.is_empty() {
                        Color::new(0.5, 0.5, 0.5, 1.0)
                    } else {
                        Color::WHITE
                    }),
            );
        }

        Ok(())
    }

    pub fn handle_menu_click(&mut self, x: f32, y: f32) -> bool {
        let menu_y = 768.0 - 40.0;
        if y >= menu_y && y <= menu_y + 40.0 {
            let button_width = 110.0;
            let button_spacing = 120.0;
            
            // Check which button was clicked
            let button_index = ((x - 10.0) / button_spacing) as i32;
            if x >= 10.0 && button_index >= 0 && button_index < 3 {
                match button_index {
                    0 => self.toggle_inventory(),
                    1 => self.toggle_skills_menu(),
                    2 => self.toggle_equipment_screen(),
                    _ => return false,
                }
                return true;
            }
        }
        false
    }

    pub fn handle_bank_click(&mut self, x: f32, y: f32, button: MouseButton, inventory: &mut Inventory, bank: &mut Bank) -> bool {
        if !self.bank_visible {
            return false;
        }

        // Check if click is on close button
        if x >= 720.0 && x <= 740.0 && y >= 15.0 && y <= 35.0 {
            self.toggle_bank();
            return true;
        }

        // Check if click is in bank window area
        if x >= 250.0 && x <= 750.0 && y >= 10.0 && y <= 610.0 {
            // Check if click is in bank slots area
            if x >= 270.0 && x <= 720.0 && y >= 50.0 && y <= 410.0 {
                let slot_x = ((x - 270.0) / 45.0).floor() as usize;
                let slot_y = ((y - 50.0) / 45.0).floor() as usize;
                let slot = slot_y * 10 + slot_x;

                if slot < 80 {
                    match button {
                        MouseButton::Left => {
                            // Withdraw item from bank to inventory
                            if let Some(item) = bank.remove_items(slot, 1) {
                                if inventory.add_item(item.clone()) {
                                    self.add_message(format!("You withdraw {}.", item.name));
                                } else {
                                    bank.add_item(item.clone()); // Put item back in bank
                                    self.add_message("Your inventory is full.".to_string());
                                }
                            }
                        }
                        MouseButton::Right => {
                            // Show context menu for all items
                            if let Some(item) = bank.get_item(slot) {
                                let mut actions = vec![
                                    ("Withdraw-1".to_string(), ContextMenuAction::WithdrawOne),
                                    ("Withdraw-10".to_string(), ContextMenuAction::WithdrawTen),
                                    ("Withdraw-100".to_string(), ContextMenuAction::WithdrawHundred),
                                    ("Withdraw-All".to_string(), ContextMenuAction::WithdrawAll),
                                    ("Withdraw-X".to_string(), ContextMenuAction::WithdrawX),
                                    ("Examine".to_string(), ContextMenuAction::Examine(format!("This is {}.", item.name))),
                                ];
                                self.context_menu.show(x, y, actions);
                                self.selected_bank_slot = Some(slot);
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            return true;
        }
        false
    }

    pub fn handle_inventory_click(&mut self, slot: usize, button: MouseButton, x: f32, y: f32, inventory: &mut Inventory) -> bool {
        if let Some(item) = inventory.get_item(slot) {
            match button {
                MouseButton::Left => {
                    self.selected_inventory_slot = Some(slot);
                    true
                }
                MouseButton::Right => {
                    if self.bank_visible {
                        let mut actions = vec![
                            ("Deposit-1".to_string(), ContextMenuAction::DepositOne),
                            ("Deposit-10".to_string(), ContextMenuAction::DepositTen),
                            ("Deposit-100".to_string(), ContextMenuAction::DepositHundred),
                            ("Deposit-All".to_string(), ContextMenuAction::DepositAll),
                            ("Deposit-X".to_string(), ContextMenuAction::DepositX),
                            ("Examine".to_string(), ContextMenuAction::Examine(format!("You have {} {}.", item.quantity, item.name))),
                        ];
                        self.context_menu.show(x, y, actions);
                        self.selected_inventory_slot = Some(slot);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn handle_context_action(&mut self, action: ContextMenuAction, inventory: &mut Inventory, bank: &mut Bank) {
        match action {
            ContextMenuAction::WithdrawOne => self.withdraw_items(1, inventory, bank),
            ContextMenuAction::WithdrawTen => self.withdraw_items(10, inventory, bank),
            ContextMenuAction::WithdrawHundred => self.withdraw_items(100, inventory, bank),
            ContextMenuAction::WithdrawAll => {
                if let Some(slot) = self.selected_bank_slot {
                    if let Some(item) = bank.get_item(slot) {
                        // For WithdrawAll, withdraw the entire stack
                        self.withdraw_items(item.quantity, inventory, bank);
                    }
                }
            }
            ContextMenuAction::WithdrawX => {
                // Show input dialog for amount
                self.show_quantity_dialog(true);
            }
            ContextMenuAction::DepositOne => self.deposit_items(1, inventory, bank),
            ContextMenuAction::DepositTen => self.deposit_items(10, inventory, bank),
            ContextMenuAction::DepositHundred => self.deposit_items(100, inventory, bank),
            ContextMenuAction::DepositAll => self.deposit_all_items(inventory, bank),
            ContextMenuAction::DepositX => {
                // Show input dialog for amount
                self.show_quantity_dialog(false);
            }
            _ => {}
        }
    }

    pub fn withdraw_items(&mut self, amount: u32, inventory: &mut Inventory, bank: &mut Bank) {
        if let Some(slot) = self.selected_bank_slot {
            if let Some(item) = bank.get_item(slot) {
                let item_name = item.name.clone();
                let withdraw_amount = amount.min(item.quantity);
                
                // For all items, we need to withdraw them one by one
                let mut items_added = 0;
                
                for _ in 0..withdraw_amount {
                    if let Some(single_item) = bank.remove_items(slot, 1) {
                        if inventory.add_item(single_item.clone()) {
                            items_added += 1;
                        } else {
                            // If inventory is full, try to put the item back in the bank
                            if items_added > 0 {
                                bank.add_item(single_item);
                                self.add_message(format!("You withdraw {} {}. Your inventory is full.", items_added, item_name));
                            } else {
                                self.add_message("Your inventory is full.".to_string());
                            }
                            return;
                        }
                    } else {
                        break; // No more items to withdraw
                    }
                }
                
                if items_added > 0 {
                    self.add_message(format!("You withdraw {} {}.", items_added, item_name));
                }
            }
        }
    }

    pub fn deposit_items(&mut self, amount: u32, inventory: &mut Inventory, bank: &mut Bank) {
        if let Some(slot) = self.selected_inventory_slot {
            if let Some(item) = inventory.get_item(slot) {
                let item_name = item.name.clone();
                let deposit_amount = amount.min(item.quantity);
                
                if let Some(deposited_item) = inventory.remove_items(slot, deposit_amount) {
                    if bank.add_item(deposited_item.clone()) {
                        self.add_message(format!("You deposit {} {}.", deposit_amount, item_name));
                    } else {
                        inventory.add_item(deposited_item); // Put items back in inventory
                        self.add_message("Your bank is full.".to_string());
                    }
                }
            }
        }
    }

    pub fn deposit_all_items(&mut self, inventory: &mut Inventory, bank: &mut Bank) {
        if let Some(slot) = self.selected_inventory_slot {
            if let Some(selected_item) = inventory.get_item(slot) {
                let item_name = selected_item.name.clone();
                let item_type = selected_item.item_type.clone();
                
                // Find all slots with the same item type
                let mut slots_to_deposit = Vec::new();
                for i in 0..inventory.get_items().len() {
                    if let Some(item) = inventory.get_item(i) {
                        if item.name == item_name && item.item_type == item_type {
                            slots_to_deposit.push(i);
                        }
                    }
                }
                
                // Deposit items starting from the highest slot index (to avoid shifting problems)
                slots_to_deposit.sort_by(|a, b| b.cmp(a));
                
                let mut total_deposited = 0;
                for slot_to_deposit in slots_to_deposit {
                    if let Some(item) = inventory.get_item(slot_to_deposit) {
                        let deposit_amount = item.quantity;
                        
                        if let Some(deposited_item) = inventory.remove_items(slot_to_deposit, deposit_amount) {
                            if bank.add_item(deposited_item.clone()) {
                                total_deposited += deposit_amount;
                            } else {
                                inventory.add_item(deposited_item); // Put items back in inventory
                                self.add_message("Your bank is full.".to_string());
                                break;
                            }
                        }
                    }
                }
                
                if total_deposited > 0 {
                    self.add_message(format!("You deposit {} {}.", total_deposited, item_name));
                }
            }
        }
    }

    pub fn show_quantity_dialog(&mut self, is_withdraw: bool) {
        self.quantity_dialog_visible = true;
        self.quantity_dialog_is_withdraw = is_withdraw;
        self.quantity_input.clear();
    }

    pub fn hide_quantity_dialog(&mut self) {
        self.quantity_dialog_visible = false;
        self.quantity_input.clear();
    }

    pub fn handle_quantity_input(&mut self, c: char) {
        if c.is_digit(10) && self.quantity_input.len() < 10 {
            self.quantity_input.push(c);
        }
    }

    pub fn handle_quantity_backspace(&mut self) {
        self.quantity_input.pop();
    }

    pub fn handle_quantity_enter(&mut self, inventory: &mut Inventory, bank: &mut Bank) {
        if let Ok(amount) = self.quantity_input.parse::<u32>() {
            if amount > 0 {
                if self.quantity_dialog_is_withdraw {
                    self.withdraw_items(amount, inventory, bank);
                } else {
                    self.deposit_items(amount, inventory, bank);
                }
            }
        }
        self.hide_quantity_dialog();
    }
}

// ... rest of the existing code ... 