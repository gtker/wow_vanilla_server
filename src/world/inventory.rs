use crate::world::database::WorldDatabase;
use crate::world::item::Item;
use wow_items::vanilla::lookup_item;
use wow_world_base::vanilla::{InventoryType, ItemSlot, StarterItem};
use wow_world_messages::vanilla::CharacterGear;

#[derive(Debug, Clone)]
pub struct Inventory {
    pub head: Option<Item>,
    pub neck: Option<Item>,
    pub shoulders: Option<Item>,
    pub shirt: Option<Item>,
    pub chest: Option<Item>,
    pub waist: Option<Item>,
    pub legs: Option<Item>,
    pub boots: Option<Item>,
    pub wrist: Option<Item>,
    pub hands: Option<Item>,
    pub ring1: Option<Item>,
    pub ring2: Option<Item>,
    pub trinket1: Option<Item>,
    pub trinket2: Option<Item>,
    pub back: Option<Item>,
    pub main_hand: Option<Item>,
    pub off_hand: Option<Item>,
    pub ranged: Option<Item>,
    pub tabard: Option<Item>,
    pub bag1: Option<Item>,
    pub bag2: Option<Item>,
    pub bag3: Option<Item>,
    pub bag4: Option<Item>,
    pub inventory0: Option<Item>,
    pub inventory1: Option<Item>,
    pub inventory2: Option<Item>,
    pub inventory3: Option<Item>,
    pub inventory4: Option<Item>,
    pub inventory5: Option<Item>,
    pub inventory6: Option<Item>,
    pub inventory7: Option<Item>,
    pub inventory8: Option<Item>,
    pub inventory9: Option<Item>,
    pub inventory10: Option<Item>,
    pub inventory11: Option<Item>,
    pub inventory12: Option<Item>,
    pub inventory13: Option<Item>,
    pub inventory14: Option<Item>,
    pub inventory15: Option<Item>,
}

impl Inventory {
    pub fn new(starter_items: &[StarterItem], db: &mut WorldDatabase) -> Self {
        let mut s = Self {
            head: None,
            neck: None,
            shoulders: None,
            shirt: None,
            chest: None,
            waist: None,
            legs: None,
            boots: None,
            wrist: None,
            hands: None,
            ring1: None,
            ring2: None,
            trinket1: None,
            trinket2: None,
            back: None,
            main_hand: None,
            off_hand: None,
            ranged: None,
            tabard: None,
            bag1: None,
            bag2: None,
            bag3: None,
            bag4: None,
            inventory0: None,
            inventory1: None,
            inventory2: None,
            inventory3: None,
            inventory4: None,
            inventory5: None,
            inventory6: None,
            inventory7: None,
            inventory8: None,
            inventory9: None,
            inventory10: None,
            inventory11: None,
            inventory12: None,
            inventory13: None,
            inventory14: None,
            inventory15: None,
        };

        for item in starter_items {
            let i = Item::new(lookup_item(item.item).unwrap(), db);
            s.set(item.ty, i);
        }

        s
    }

    pub fn equipment(&self) -> [Option<&Item>; 19] {
        [
            self.head.as_ref(),
            self.neck.as_ref(),
            self.shoulders.as_ref(),
            self.shirt.as_ref(),
            self.chest.as_ref(),
            self.waist.as_ref(),
            self.legs.as_ref(),
            self.boots.as_ref(),
            self.wrist.as_ref(),
            self.hands.as_ref(),
            self.ring1.as_ref(),
            self.ring2.as_ref(),
            self.trinket1.as_ref(),
            self.trinket2.as_ref(),
            self.back.as_ref(),
            self.main_hand.as_ref(),
            self.off_hand.as_ref(),
            self.ranged.as_ref(),
            self.tabard.as_ref(),
        ]
    }

    pub fn to_character_gear(&self) -> [CharacterGear; 19] {
        let mut gear = [CharacterGear::default(); 19];

        for (i, item) in self.equipment().iter().enumerate() {
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

    pub fn get(&self, item_slot: ItemSlot) -> Option<&Item> {
        match item_slot {
            ItemSlot::Head => self.head.as_ref(),
            ItemSlot::Neck => self.neck.as_ref(),
            ItemSlot::Shoulders => self.shoulders.as_ref(),
            ItemSlot::Shirt => self.shirt.as_ref(),
            ItemSlot::Chest => self.chest.as_ref(),
            ItemSlot::Waist => self.waist.as_ref(),
            ItemSlot::Legs => self.legs.as_ref(),
            ItemSlot::Boots => self.boots.as_ref(),
            ItemSlot::Wrist => self.wrist.as_ref(),
            ItemSlot::Hands => self.hands.as_ref(),
            ItemSlot::Ring1 => self.ring1.as_ref(),
            ItemSlot::Ring2 => self.ring2.as_ref(),
            ItemSlot::Trinket1 => self.trinket1.as_ref(),
            ItemSlot::Trinket2 => self.trinket2.as_ref(),
            ItemSlot::Back => self.back.as_ref(),
            ItemSlot::MainHand => self.main_hand.as_ref(),
            ItemSlot::OffHand => self.off_hand.as_ref(),
            ItemSlot::RangedOrRelic => self.ranged.as_ref(),
            ItemSlot::Tabard => self.tabard.as_ref(),
            ItemSlot::Bag1 => self.bag1.as_ref(),
            ItemSlot::Bag2 => self.bag2.as_ref(),
            ItemSlot::Bag3 => self.bag3.as_ref(),
            ItemSlot::Bag4 => self.bag4.as_ref(),
            ItemSlot::Inventory0 => self.inventory0.as_ref(),
            ItemSlot::Inventory1 => self.inventory1.as_ref(),
            ItemSlot::Inventory2 => self.inventory2.as_ref(),
            ItemSlot::Inventory3 => self.inventory3.as_ref(),
            ItemSlot::Inventory4 => self.inventory4.as_ref(),
            ItemSlot::Inventory5 => self.inventory5.as_ref(),
            ItemSlot::Inventory6 => self.inventory6.as_ref(),
            ItemSlot::Inventory7 => self.inventory7.as_ref(),
            ItemSlot::Inventory8 => self.inventory8.as_ref(),
            ItemSlot::Inventory9 => self.inventory9.as_ref(),
            ItemSlot::Inventory10 => self.inventory10.as_ref(),
            ItemSlot::Inventory11 => self.inventory11.as_ref(),
            ItemSlot::Inventory12 => self.inventory12.as_ref(),
            ItemSlot::Inventory13 => self.inventory13.as_ref(),
            ItemSlot::Inventory14 => self.inventory14.as_ref(),
            ItemSlot::Inventory15 => self.inventory15.as_ref(),
        }
    }

    pub fn set(&mut self, item_slot: ItemSlot, item: Item) {
        match item_slot {
            ItemSlot::Head => self.head = Some(item),
            ItemSlot::Neck => self.neck = Some(item),
            ItemSlot::Shoulders => self.shoulders = Some(item),
            ItemSlot::Shirt => self.shirt = Some(item),
            ItemSlot::Chest => self.chest = Some(item),
            ItemSlot::Waist => self.waist = Some(item),
            ItemSlot::Legs => self.legs = Some(item),
            ItemSlot::Boots => self.boots = Some(item),
            ItemSlot::Wrist => self.wrist = Some(item),
            ItemSlot::Hands => self.hands = Some(item),
            ItemSlot::Ring1 => self.ring1 = Some(item),
            ItemSlot::Ring2 => self.ring2 = Some(item),
            ItemSlot::Trinket1 => self.trinket1 = Some(item),
            ItemSlot::Trinket2 => self.trinket2 = Some(item),
            ItemSlot::Back => self.back = Some(item),
            ItemSlot::MainHand => self.main_hand = Some(item),
            ItemSlot::OffHand => self.off_hand = Some(item),
            ItemSlot::RangedOrRelic => self.ranged = Some(item),
            ItemSlot::Tabard => self.tabard = Some(item),
            ItemSlot::Bag1 => self.bag1 = Some(item),
            ItemSlot::Bag2 => self.bag2 = Some(item),
            ItemSlot::Bag3 => self.bag3 = Some(item),
            ItemSlot::Bag4 => self.bag4 = Some(item),
            ItemSlot::Inventory0 => self.inventory0 = Some(item),
            ItemSlot::Inventory1 => self.inventory1 = Some(item),
            ItemSlot::Inventory2 => self.inventory2 = Some(item),
            ItemSlot::Inventory3 => self.inventory3 = Some(item),
            ItemSlot::Inventory4 => self.inventory4 = Some(item),
            ItemSlot::Inventory5 => self.inventory5 = Some(item),
            ItemSlot::Inventory6 => self.inventory6 = Some(item),
            ItemSlot::Inventory7 => self.inventory7 = Some(item),
            ItemSlot::Inventory8 => self.inventory8 = Some(item),
            ItemSlot::Inventory9 => self.inventory9 = Some(item),
            ItemSlot::Inventory10 => self.inventory10 = Some(item),
            ItemSlot::Inventory11 => self.inventory11 = Some(item),
            ItemSlot::Inventory12 => self.inventory12 = Some(item),
            ItemSlot::Inventory13 => self.inventory13 = Some(item),
            ItemSlot::Inventory14 => self.inventory14 = Some(item),
            ItemSlot::Inventory15 => self.inventory15 = Some(item),
        }
    }
}
