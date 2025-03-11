use serde::{Serialize, Deserialize};
use crate::inventory::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
    items: Vec<Option<Item>>,
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
        // If the item is stackable, try to stack it with existing items first
        if item.is_stackable() {
            for existing_item in self.items.iter_mut().filter_map(|x| x.as_mut()) {
                if existing_item.name == item.name {
                    existing_item.quantity += item.quantity;
                    return true;
                }
            }
        }

        // If we couldn't stack it (or it's not stackable), find an empty slot
        if let Some(empty_slot) = self.items.iter_mut().find(|slot| slot.is_none()) {
            *empty_slot = Some(item);
            true
        } else {
            false
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

    pub fn get_item(&self, index: usize) -> Option<&Item> {
        self.items.get(index).and_then(|opt| opt.as_ref())
    }

    pub fn get_items(&self) -> &Vec<Option<Item>> {
        &self.items
    }
} 