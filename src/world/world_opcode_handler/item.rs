use crate::world::database::WorldDatabase;
use crate::world::world::client::Client;
use wow_world_base::vanilla::{
    BagFamily, NewItemChatAlert, NewItemCreationType, NewItemSource, ObjectType,
};
use wow_world_messages::vanilla::{
    MovementBlock, MovementBlock_UpdateFlag, Object, UpdateItemBuilder, UpdatePlayerBuilder,
    SMSG_ITEM_PUSH_RESULT, SMSG_UPDATE_OBJECT,
};
use wow_world_messages::Guid;

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub item: &'static wow_items::vanilla::Item,
    pub guid: Guid,
    pub amount: u8,
    pub creator: Guid,
}

impl Item {
    pub fn new(
        item: &'static wow_items::vanilla::Item,
        creator: Guid,
        amount: u8,
        db: &mut WorldDatabase,
    ) -> Self {
        Self {
            item,
            guid: db.new_guid().into(),
            amount,
            creator,
        }
    }

    pub fn to_create_item_object(&self, item_owner: Guid) -> Object {
        let object_type = match self.item.bag_family() {
            BagFamily::None => ObjectType::Item,
            _ => ObjectType::Container,
        };

        Object::CreateObject {
            guid3: self.guid,
            mask2: UpdateItemBuilder::new()
                .set_object_guid(self.guid)
                .set_object_entry(self.item.entry() as i32)
                .set_object_scale_x(1.0)
                .set_item_owner(item_owner)
                .set_item_contained(item_owner)
                .set_item_stack_count(self.amount as i32)
                .set_item_durability(self.item.max_durability())
                .set_item_maxdurability(self.item.max_durability())
                .set_item_creator(self.creator)
                .set_item_stack_count(self.amount as i32)
                .finalize()
                .into(),
            movement2: MovementBlock {
                update_flag: MovementBlock_UpdateFlag::empty(),
            },
            object_type,
        }
    }
}

pub(crate) async fn award_item(item: Item, client: &mut Client, clients: &mut [Client]) {
    let item_slot = client
        .character_mut()
        .inventory
        .insert_into_first_slot(item);
    let Some(item_slot) = item_slot else {
        client
            .send_system_message("Unable to add item. No free slots available.")
            .await;
        return;
    };

    client
        .send_opcode(
            &SMSG_UPDATE_OBJECT {
                has_transport: 0,
                objects: vec![
                    item.to_create_item_object(client.character().guid),
                    Object::Values {
                        guid1: client.character().guid,
                        mask1: UpdatePlayerBuilder::new()
                            .set_player_field_inv(item_slot, item.guid)
                            .finalize()
                            .into(),
                    },
                ],
            }
            .into(),
        )
        .await;

    let item_push_result = SMSG_ITEM_PUSH_RESULT {
        guid: client.character().guid,
        source: NewItemSource::Looted,
        creation_type: NewItemCreationType::Created,
        alert_chat: NewItemChatAlert::Show,
        bag_slot: 0xff,
        item_slot: item_slot.as_int() as u32,
        item: item.item.entry(),
        item_suffix_factor: 0,
        item_random_property_id: 0,
        item_count: item.amount.into(),
    };

    client.send_opcode(&item_push_result.into()).await;

    for c in clients {
        c.send_opcode(&item_push_result.into()).await;
    }
}
pub mod messages {

    use wow_items::vanilla::Item;
    use wow_world_base::vanilla::*;
    use wow_world_messages::vanilla::{
        SMSG_ITEM_QUERY_SINGLE_RESPONSE_found, SMSG_ITEM_NAME_QUERY_RESPONSE,
        SMSG_ITEM_QUERY_SINGLE_RESPONSE,
    };

    /// Convert an [`Item`] to a [`SMSG_ITEM_QUERY_SINGLE_RESPONSE`].
    ///
    /// The message is just a tedious listing of [`Item`] fields with no
    /// potential deviations so it has been upstreamed.
    pub fn item_to_query_response(v: &Item) -> SMSG_ITEM_QUERY_SINGLE_RESPONSE {
        let mut spells = [wow_world_base::vanilla::ItemSpells::default(); 5];
        for (i, spell) in v.spells_array().iter().enumerate() {
            spells[i] = wow_world_base::vanilla::ItemSpells {
                spell: spell.spell as u32,
                spell_trigger: spell.spell_trigger,
                spell_charges: spell.spell_charges,
                spell_cooldown: spell.spell_cooldown,
                spell_category: spell.spell_category as u32,
                spell_category_cooldown: spell.spell_category_cooldown,
            };
        }

        SMSG_ITEM_QUERY_SINGLE_RESPONSE {
            item: v.entry(),
            found: Some(SMSG_ITEM_QUERY_SINGLE_RESPONSE_found {
                class_and_sub_class: v.class_and_sub_class(),
                name1: v.name().to_string(),
                name2: String::default(),
                name3: String::default(),
                name4: String::default(),
                display_id: v.display_id(),
                quality: v.quality(),
                flags: v.flags(),
                buy_price: v.buy_price(),
                sell_price: v.sell_price(),
                inventory_type: v.inventory_type(),
                allowed_class: v.allowed_class(),
                allowed_race: v.allowed_race(),
                item_level: Level::new(v.item_level() as u8),
                required_level: Level::new(v.required_level() as u8),
                required_skill: v.required_skill(),
                required_skill_rank: v.required_skill_rank() as u32,
                required_spell: v.required_spell() as u32,
                required_honor_rank: v.required_honor_rank() as u32,
                required_city_rank: v.required_city_rank() as u32,
                required_faction: v.required_faction(),
                required_faction_rank: v.required_reputation_rank() as u32,
                max_count: v.max_count() as u32,
                stackable: v.stackable() as u32,
                container_slots: v.container_slots() as u32,
                stats: get_item_stats(v),
                damages: v.damages_array(),
                armor: v.armor(),
                holy_resistance: v.holy_res(),
                fire_resistance: v.fire_res(),
                nature_resistance: v.nature_res(),
                frost_resistance: v.frost_res(),
                shadow_resistance: v.shadow_res(),
                arcane_resistance: v.arcane_res(),
                delay: v.delay() as u32,
                ammo_type: v.ammo_type() as u32,
                ranged_range_modification: v.ranged_mod_range(),
                spells,
                bonding: v.bonding(),
                description: v.description().to_string(),
                page_text: v.page_text() as u32,
                language: v.language(),
                page_text_material: v.page_text_material(),
                start_quest: v.start_quest() as u32,
                lock_id: v.lock_id() as u32,
                material: v.material() as u32,
                sheathe_type: v.sheathe_type(),
                random_property: v.random_property() as u32,
                block: v.block() as u32,
                item_set: v.item_set(),
                max_durability: v.max_durability() as u32,
                area: v.area(),
                map: v.map(),
                bag_family: v.bag_family(),
            }),
        }
    }

    fn get_item_stats(v: &Item) -> [ItemStat; 10] {
        let mut stats = [ItemStat::default(); 10];
        stats[0].stat_type = ItemStatType::Mana;
        stats[0].value = v.mana();
        stats[1].stat_type = ItemStatType::Health;
        stats[1].value = v.health();
        stats[2].stat_type = ItemStatType::Agility;
        stats[2].value = v.agility();
        stats[3].stat_type = ItemStatType::Strength;
        stats[3].value = v.strength();
        stats[4].stat_type = ItemStatType::Intellect;
        stats[4].value = v.intellect();
        stats[5].stat_type = ItemStatType::Spirit;
        stats[5].value = v.spirit();
        stats[6].stat_type = ItemStatType::Stamina;
        stats[6].value = v.stamina();
        stats
    }

    /// Convert an [`Item`] to a [`SMSG_ITEM_NAME_QUERY_RESPONSE`].
    ///
    /// The message is just a listing of [`Item`] fields with no
    /// potential deviations so it has been upstreamed.
    pub fn item_to_name_query_response(v: &Item) -> SMSG_ITEM_NAME_QUERY_RESPONSE {
        SMSG_ITEM_NAME_QUERY_RESPONSE {
            item: v.entry(),
            item_name: v.name().to_string(),
        }
    }
}
