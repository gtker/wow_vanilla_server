use wow_world_base::wrath::item::Item;
use wow_world_base::wrath::InventoryType;
use wow_world_messages::wrath::{
    ItemClass, ItemDamageType, ItemQuality, ItemSocket, ItemSpells, ItemStat,
    SMSG_ITEM_QUERY_SINGLE_RESPONSE_found, SMSG_ITEM_QUERY_SINGLE_RESPONSE,
};

pub fn item_to_response(item: &'static Item) -> SMSG_ITEM_QUERY_SINGLE_RESPONSE {
    SMSG_ITEM_QUERY_SINGLE_RESPONSE {
        item: item.entry as u32,
        found: Some(SMSG_ITEM_QUERY_SINGLE_RESPONSE_found {
            item_class: ItemClass::try_from(item.class as u8).unwrap(),
            item_sub_class: item.subclass as u32,
            unknown1: 0,
            name1: item.name.to_string(),
            name2: "".to_string(),
            name3: "".to_string(),
            name4: "".to_string(),
            item_display_info: item.displayid as u32,
            quality: ItemQuality::try_from(item.quality as u8).unwrap(),
            flags: item.flags as u32,
            flags2: item.flags2 as u32,
            buy_price: item.buy_price as u32,
            sell_price: item.sell_price as u32,
            inventory_type: InventoryType::try_from(item.inventory_type as u8).unwrap(),
            allowed_class: 0,
            allowed_race: 0,
            item_level: 0,
            required_level: item.required_level as u32,
            required_skill: Default::default(),
            required_skill_rank: 0,
            required_spell: 0,
            required_honor_rank: 0,
            required_city_rank: 0,
            required_reputation_faction: 0,
            required_reputation_rank: 0,
            max_count: 0,
            stackable: 0,
            container_slots: 0,
            amount_of_stats: 1,
            stats: vec![ItemStat {
                item_stat_type: 0,
                item_stat_value: 100,
            }],
            scaling_stats_entry: 0,
            scaling_stats_flag: 0,
            damages: [ItemDamageType::default(); 2],
            armor: 0,
            holy_resistance: 0,
            fire_resistance: 0,
            nature_resistance: 0,
            frost_resistance: 0,
            shadow_resistance: 0,
            arcane_resistance: 0,
            delay: 0,
            ammo_type: 0,
            ranged_range_modification: 0.0,
            spells: [ItemSpells::default(); 5],
            bonding: 0,
            description: "".to_string(),
            page_text: 0,
            language_id: 0,
            page_material: 0,
            start_quest: 0,
            lock_id: 0,
            material: 0,
            sheath: 0,
            random_property: 0,
            block: 0,
            item_set: 0,
            max_durability: 0,
            area: Default::default(),
            map: Default::default(),
            bag_family: 0,
            totem_category: 0,
            sockets: [ItemSocket::default(); 3],
            socket_bonus: 0,
            gem_properties: 0,
            required_disenchant_skill: 0,
            armor_damage_modifier: 0.0,
            duration_in_seconds: 0,
            item_limit_category: 0,
            holiday_id: 0,
        }),
    }
}
