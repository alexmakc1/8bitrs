use serde::{Serialize, Deserialize};
use crate::inventory::{Item, ItemType};

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
        // Stack all items in the bank, regardless of whether they're marked as stackable
        for existing_item in self.items.iter_mut().filter_map(|x| x.as_mut()) {
            if existing_item.name == item.name && existing_item.item_type == item.item_type {
                existing_item.quantity += item.quantity;
                return true;
            }
        }

        // If we couldn't stack it, find an empty slot
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
            // Always remove one item at a time
            if amount >= 1 {
                // Create a copy of the item's data
                let name = item.name.clone();
                let item_type = item.item_type.clone();
                
                // Reduce the quantity in the bank
                if item.quantity <= 1 {
                    // Last item, remove it completely
                    let mut removed_item = self.items[index].take().unwrap();
                    // Force the item to be unstackable
                    removed_item.make_unstackable();
                    removed_item.quantity = 1;
                    return Some(removed_item);
                } else {
                    // Reduce the stack by 1
                    item.quantity -= 1;
                    
                    // Create a new item that is guaranteed to be unstackable
                    let mut new_item = Item {
                        name,
                        item_type,
                        stackable: false,
                        quantity: 1,
                    };
                    new_item.make_unstackable();
                    return Some(new_item);
                }
            }
            return None;
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