use serde::{Serialize, Deserialize};
use crate::inventory::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankSlot {
    pub item: Item,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
    items: Vec<Option<BankSlot>>,
    capacity: usize,
}

impl Bank {
    pub fn new(capacity: usize) -> Self {
        Bank {
            items: vec![None; capacity],
            capacity,
        }
    }

    pub fn add_item(&mut self, item: Item) -> bool {
        // First try to stack with existing items
        for slot in self.items.iter_mut() {
            if let Some(bank_slot) = slot {
                if bank_slot.item.name == item.name {
                    bank_slot.quantity += 1;
                    return true;
                }
            }
        }

        // If no stack found, find empty slot
        if let Some(empty_slot) = self.items.iter_mut().find(|slot| slot.is_none()) {
            *empty_slot = Some(BankSlot {
                item,
                quantity: 1,
            });
            true
        } else {
            false
        }
    }

    pub fn get_items(&self) -> &Vec<Option<BankSlot>> {
        &self.items
    }

    pub fn get_item(&self, slot: usize) -> Option<&BankSlot> {
        self.items.get(slot)?.as_ref()
    }

    pub fn remove_item(&mut self, slot: usize) -> Option<Item> {
        if let Some(Some(bank_slot)) = self.items.get_mut(slot) {
            if bank_slot.quantity > 1 {
                bank_slot.quantity -= 1;
                Some(bank_slot.item.clone())
            } else {
                // Remove the last item
                let slot_value = self.items[slot].take();
                slot_value.map(|s| s.item)
            }
        } else {
            None
        }
    }

    pub fn is_full(&self) -> bool {
        !self.items.iter().any(|slot| slot.is_none())
    }
} 