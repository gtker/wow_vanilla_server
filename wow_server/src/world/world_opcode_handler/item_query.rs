use wow_world_base::wrath::item::Item;
use wow_world_base::wrath::{InventoryType, Skill};
use wow_world_messages::wrath::{
    ItemClass, ItemDamageType, ItemQuality, ItemSocket, ItemSpells, ItemStat,
    SMSG_ITEM_QUERY_SINGLE_RESPONSE_found, SMSG_ITEM_QUERY_SINGLE_RESPONSE,
};

pub fn item_to_response(item: &'static Item) -> SMSG_ITEM_QUERY_SINGLE_RESPONSE {
    let stats = {
        let mut v = Vec::new();

        if item.stat_type1 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type1 as u32,
                item_stat_value: item.stat_value1,
            });
        }
        if item.stat_type2 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type2 as u32,
                item_stat_value: item.stat_value2,
            });
        }
        if item.stat_type3 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type3 as u32,
                item_stat_value: item.stat_value3,
            });
        }
        if item.stat_type4 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type4 as u32,
                item_stat_value: item.stat_value4,
            });
        }
        if item.stat_type5 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type5 as u32,
                item_stat_value: item.stat_value5,
            });
        }
        if item.stat_type6 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type6 as u32,
                item_stat_value: item.stat_value6,
            });
        }
        if item.stat_type7 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type7 as u32,
                item_stat_value: item.stat_value7,
            });
        }
        if item.stat_type8 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type8 as u32,
                item_stat_value: item.stat_value8,
            });
        }
        if item.stat_type9 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type9 as u32,
                item_stat_value: item.stat_value9,
            });
        }
        if item.stat_type10 != 0 {
            v.push(ItemStat {
                item_stat_type: item.stat_type10 as u32,
                item_stat_value: item.stat_value10,
            });
        }

        v
    };

    let spells = {
        let mut v = [ItemSpells::default(); 5];

        if item.spell_id_1 != 0 {
            v[0].spell = item.spell_id_1 as u32;
            v[0].spell_trigger = item.spell_trigger_1 as u32;
            v[0].spell_charges = item.spell_charges_1;
            v[0].spell_cooldown = item.spell_cooldown_1;
            v[0].spell_category = item.spell_category_1 as u32;
            v[0].spell_category_cooldown = item.spell_category_cooldown_1;
        }

        if item.spell_id_2 != 0 {
            v[1].spell = item.spell_id_2 as u32;
            v[1].spell_trigger = item.spell_trigger_2 as u32;
            v[1].spell_charges = item.spell_charges_2;
            v[1].spell_cooldown = item.spell_cooldown_2;
            v[1].spell_category = item.spell_category_2 as u32;
            v[1].spell_category_cooldown = item.spell_category_cooldown_2;
        }

        if item.spell_id_3 != 0 {
            v[2].spell = item.spell_id_3 as u32;
            v[2].spell_trigger = item.spell_trigger_3 as u32;
            v[2].spell_charges = item.spell_charges_3;
            v[2].spell_cooldown = item.spell_cooldown_3;
            v[2].spell_category = item.spell_category_3 as u32;
            v[2].spell_category_cooldown = item.spell_category_cooldown_3;
        }

        if item.spell_id_4 != 0 {
            v[3].spell = item.spell_id_4 as u32;
            v[3].spell_trigger = item.spell_trigger_4 as u32;
            v[3].spell_charges = item.spell_charges_4;
            v[3].spell_cooldown = item.spell_cooldown_4;
            v[3].spell_category = item.spell_category_4 as u32;
            v[3].spell_category_cooldown = item.spell_category_cooldown_4;
        }

        if item.spell_id_5 != 0 {
            v[4].spell = item.spell_id_5 as u32;
            v[4].spell_trigger = item.spell_trigger_5 as u32;
            v[4].spell_charges = item.spell_charges_5;
            v[4].spell_cooldown = item.spell_cooldown_5;
            v[4].spell_category = item.spell_category_5 as u32;
            v[4].spell_category_cooldown = item.spell_category_cooldown_5;
        }

        v
    };

    SMSG_ITEM_QUERY_SINGLE_RESPONSE {
        item: item.entry as u32,
        found: Some(SMSG_ITEM_QUERY_SINGLE_RESPONSE_found {
            item_class: ItemClass::try_from(item.class as u8).unwrap(),
            item_sub_class: item.subclass as u32,
            unknown1: item.unk0 as u32,
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
            allowed_class: item.allowed_class as u32,
            allowed_race: item.allowed_race as u32,
            item_level: item.item_level as u32,
            required_level: item.required_level as u32,
            required_skill: Skill::try_from(item.required_skill as u16).unwrap(),
            required_skill_rank: item.required_skill_rank as u32,
            required_spell: item.required_spell as u32,
            required_honor_rank: item.required_honor_rank as u32,
            required_city_rank: item.required_city_rank as u32,
            required_reputation_faction: item.required_reputation_faction as u32,
            required_reputation_rank: item.required_reputation_rank as u32,
            max_count: item.max_count as u32,
            stackable: item.stackable as u32,
            container_slots: item.container_slots as u32,
            stats,
            scaling_stats_entry: 0,
            scaling_stats_flag: 0,
            damages: [ItemDamageType::default(); 2],
            armor: item.armor,
            holy_resistance: item.holy_res,
            fire_resistance: item.fire_res,
            nature_resistance: item.nature_res,
            frost_resistance: item.frost_res,
            shadow_resistance: item.shadow_res,
            arcane_resistance: item.arcane_res,
            delay: item.delay as u32,
            ammo_type: item.ammo_type as u32,
            ranged_range_modification: item.ranged_mod_range,
            spells,
            bonding: item.bonding as u32,
            description: item.description.to_string(),
            page_text: item.page_text as u32,
            language_id: item.language_id as u32,
            page_material: item.page_material as u32,
            start_quest: item.start_quest as u32,
            lock_id: item.lock_id as u32,
            material: item.material as u32,
            sheath: item.sheath as u32,
            random_property: item.random_property as u32,
            block: item.block as u32,
            item_set: item.itemset as u32,
            max_durability: item.max_durability as u32,
            area: Default::default(),
            map: Default::default(),
            bag_family: item.bag_family as u32,
            totem_category: item.totem_category as u32,
            sockets: [ItemSocket::default(); 3],
            socket_bonus: item.socket_bonus as u32,
            gem_properties: item.gem_properties as u32,
            required_disenchant_skill: item.required_disenchant_skill as u32,
            armor_damage_modifier: 0.0,
            duration_in_seconds: 0,
            item_limit_category: 0,
            holiday_id: 0,
        }),
    }
}
