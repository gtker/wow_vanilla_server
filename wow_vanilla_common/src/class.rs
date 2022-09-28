use wow_world_base::vanilla::{Class, Gender, Power, Race};

pub fn get_display_id_for_player(race: Race, gender: Gender) -> i32 {
    let race = match race {
        Race::Human => 49,
        Race::Orc => 51,
        Race::Dwarf => 53,
        Race::NightElf => 55,
        Race::Undead => 57,
        Race::Tauren => 59,
        Race::Gnome => 1563,
        Race::Troll => 1478,
        Race::Goblin => 1140,
    };

    let gender = match gender {
        Gender::Male => 0,
        Gender::Female => 1,
        Gender::None => 0,
    };

    race + gender
}

pub fn get_power_for_class(class: Class) -> Power {
    match class {
        Class::Warrior => Power::Rage,
        Class::Rogue => Power::Energy,
        Class::Paladin
        | Class::Hunter
        | Class::Priest
        | Class::Shaman
        | Class::Mage
        | Class::Warlock
        | Class::Druid => Power::Mana,
    }
}
