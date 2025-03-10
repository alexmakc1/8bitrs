use crate::inventory::{Item, ItemType, ArmorSlot};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    weapon: Option<Item>,
    head: Option<Item>,
    body: Option<Item>,
    legs: Option<Item>,
}

impl Equipment {
    pub fn new() -> Self {
        Equipment {
            weapon: None,
            head: None,
            body: None,
            legs: None,
        }
    }

    pub fn equip_weapon(&mut self, item: Item) -> Option<Item> {
        let old_weapon = self.weapon.take();
        self.weapon = Some(item);
        old_weapon
    }

    pub fn equip_armor(&mut self, item: Item) -> Option<Item> {
        if let ItemType::Armor(stats) = &item.item_type {
            match stats.slot {
                ArmorSlot::Head => {
                    let old_item = self.head.take();
                    self.head = Some(item);
                    old_item
                }
                ArmorSlot::Body => {
                    let old_item = self.body.take();
                    self.body = Some(item);
                    old_item
                }
                ArmorSlot::Legs => {
                    let old_item = self.legs.take();
                    self.legs = Some(item);
                    old_item
                }
            }
        } else {
            None
        }
    }

    pub fn unequip_weapon(&mut self) -> Option<Item> {
        self.weapon.take()
    }

    pub fn unequip_armor(&mut self, slot: ArmorSlot) -> Option<Item> {
        match slot {
            ArmorSlot::Head => self.head.take(),
            ArmorSlot::Body => self.body.take(),
            ArmorSlot::Legs => self.legs.take(),
        }
    }

    pub fn get_weapon(&self) -> Option<&Item> {
        self.weapon.as_ref()
    }

    pub fn get_armor(&self, slot: &ArmorSlot) -> Option<&Item> {
        match slot {
            ArmorSlot::Head => self.head.as_ref(),
            ArmorSlot::Body => self.body.as_ref(),
            ArmorSlot::Legs => self.legs.as_ref(),
        }
    }

    pub fn get_total_attack_bonus(&self) -> i32 {
        if let Some(weapon) = &self.weapon {
            if let ItemType::Weapon(stats) = &weapon.item_type {
                return stats.attack_bonus;
            }
        }
        0
    }

    pub fn get_total_strength_bonus(&self) -> i32 {
        if let Some(weapon) = &self.weapon {
            if let ItemType::Weapon(stats) = &weapon.item_type {
                return stats.strength_bonus;
            }
        }
        0
    }

    pub fn get_total_defense_bonus(&self) -> i32 {
        let mut total = 0;

        // Add up defense bonuses from all armor pieces
        for armor in [&self.head, &self.body, &self.legs].iter() {
            if let Some(item) = armor {
                if let ItemType::Armor(stats) = &item.item_type {
                    total += stats.defense_bonus;
                }
            }
        }

        total
    }
} 