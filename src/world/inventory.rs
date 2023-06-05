use crate::world::database::WorldDatabase;
use crate::world::item::Item;
use wow_items::vanilla::lookup_item;
use wow_world_base::vanilla::{Guid, ItemSlot, StarterItem};
use wow_world_messages::vanilla::CharacterGear;

const AMOUNT_OF_SLOTS: usize = 113;

#[derive(Debug, Clone)]
pub struct Inventory {
    pub slots: [Option<Item>; AMOUNT_OF_SLOTS],
}

impl Inventory {
    pub fn new(starter_items: &[StarterItem], db: &mut WorldDatabase) -> Self {
        let slots = [(); AMOUNT_OF_SLOTS].map(|()| None);
        let mut s = Self { slots };

        for item in starter_items {
            let i = Item::new(
                lookup_item(item.item).unwrap(),
                Guid::zero(),
                item.amount,
                db,
            );
            s.set(item.ty, i);
        }

        s
    }

    pub fn swap(&mut self, source: ItemSlot, destination: ItemSlot) {
        let source_temp = self.take(source);
        let dest_temp = self.take(destination);

        *self.get_mut(source) = dest_temp;
        *self.get_mut(destination) = source_temp;
    }

    pub fn insert_into_first_slot(&mut self, item: Item) -> Option<ItemSlot> {
        let bag_start: usize = ItemSlot::Inventory0.as_int().into();
        let bag_end: usize = ItemSlot::Inventory15.as_int().into();
        let slots = &mut self.slots[bag_start..=bag_end];

        for (i, slot) in slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(item);

                let slot = bag_start + i;
                return Some(ItemSlot::try_from(slot as u8).unwrap());
            }
        }

        None
    }

    pub fn all_slots(&self) -> [(Option<&Item>, ItemSlot); AMOUNT_OF_SLOTS] {
        let mut slots = [(); AMOUNT_OF_SLOTS].map(|()| (None, ItemSlot::default()));

        for (i, slot) in self.slots.iter().enumerate() {
            slots[i] = (slot.as_ref(), ItemSlot::try_from(i as u8).unwrap());
        }

        slots
    }

    pub fn equipment(&self) -> [(Option<&Item>, ItemSlot); 19] {
        let mut slots = [(); 19].map(|()| (None, ItemSlot::default()));

        let inventory_start: usize = ItemSlot::Head.as_int().into();
        let inventory_end: usize = ItemSlot::Tabard.as_int().into();

        for (i, slot) in self.slots[inventory_start..=inventory_end]
            .iter()
            .enumerate()
        {
            slots[i] = (slot.as_ref(), ItemSlot::try_from(i as u8).unwrap());
        }

        slots
    }

    pub fn to_character_gear(&self) -> [CharacterGear; 19] {
        let mut gear = [CharacterGear::default(); 19];

        for (i, (item, _)) in self.equipment().iter().enumerate() {
            if let Some(item) = item {
                let g = CharacterGear {
                    equipment_display_id: item.item.display_id(),
                    inventory_type: item.item.inventory_type(),
                };
                gear[i] = g;
            } else {
                gear[i] = CharacterGear {
                    equipment_display_id: 0,
                    inventory_type: Default::default(),
                };
            }
        }

        gear
    }

    pub fn set(&mut self, item_slot: ItemSlot, item: Item) {
        *self.get_mut(item_slot) = Some(item);
    }

    pub fn clear(&mut self, item_slot: ItemSlot) {
        *self.get_mut(item_slot) = None;
    }

    pub fn take(&mut self, item_slot: ItemSlot) -> Option<Item> {
        self.get_mut(item_slot).take()
    }

    pub fn get(&self, item_slot: ItemSlot) -> Option<&Item> {
        self.inner_get(item_slot).as_ref()
    }

    fn inner_get(&self, item_slot: ItemSlot) -> &Option<Item> {
        &self.slots[item_slot.as_int() as usize]
    }

    fn get_mut(&mut self, item_slot: ItemSlot) -> &mut Option<Item> {
        &mut self.slots[item_slot.as_int() as usize]
    }
}
