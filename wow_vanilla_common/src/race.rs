use wow_world_base::vanilla::{Class, Gender, Race};

pub fn get_race_scale(race: Race, gender: Gender) -> f32 {
    match race {
        Race::Human
        | Race::Orc
        | Race::Dwarf
        | Race::NightElf
        | Race::Undead
        | Race::Gnome
        | Race::Troll
        | Race::Goblin => 1.0,
        Race::Tauren => match gender {
            Gender::Male => 1.35,
            Gender::Female => 1.25,
            Gender::None => 1.0,
        },
    }
}

pub fn character_race_class_combo_is_valid(race: Race, class: Class) -> bool {
    match (race, class) {
        (Race::Dwarf, Class::Hunter)
        | (Race::Dwarf, Class::Paladin)
        | (Race::Dwarf, Class::Priest)
        | (Race::Dwarf, Class::Rogue)
        | (Race::Dwarf, Class::Warrior)
        | (Race::Gnome, Class::Mage)
        | (Race::Gnome, Class::Rogue)
        | (Race::Gnome, Class::Warlock)
        | (Race::Gnome, Class::Warrior)
        | (Race::Human, Class::Mage)
        | (Race::Human, Class::Paladin)
        | (Race::Human, Class::Priest)
        | (Race::Human, Class::Rogue)
        | (Race::Human, Class::Warlock)
        | (Race::Human, Class::Warrior)
        | (Race::NightElf, Class::Druid)
        | (Race::NightElf, Class::Hunter)
        | (Race::NightElf, Class::Priest)
        | (Race::NightElf, Class::Rogue)
        | (Race::NightElf, Class::Warrior)
        | (Race::Orc, Class::Hunter)
        | (Race::Orc, Class::Rogue)
        | (Race::Orc, Class::Shaman)
        | (Race::Orc, Class::Warlock)
        | (Race::Orc, Class::Warrior)
        | (Race::Tauren, Class::Druid)
        | (Race::Tauren, Class::Hunter)
        | (Race::Tauren, Class::Shaman)
        | (Race::Tauren, Class::Warrior)
        | (Race::Troll, Class::Hunter)
        | (Race::Troll, Class::Mage)
        | (Race::Troll, Class::Priest)
        | (Race::Troll, Class::Rogue)
        | (Race::Troll, Class::Shaman)
        | (Race::Troll, Class::Warrior)
        | (Race::Undead, Class::Mage)
        | (Race::Undead, Class::Priest)
        | (Race::Undead, Class::Rogue)
        | (Race::Undead, Class::Warlock)
        | (Race::Undead, Class::Warrior) => true,
        (_, _) => false,
    }
}

pub fn character_features_are_valid(
    race: Race,
    gender: Gender,
    skin: u8,
    facial_hair: u8,
    face: u8,
    hair_color: u8,
    hair_style: u8,
) -> bool {
    match (race, gender) {
        // Goblin and None are never valid for player characters
        (Race::Goblin, _) | (_, Gender::None) => return false,
        _ => {}
    }

    // Valid skin colors are all between 0 and a race/gender specific number
    let max_skin_color = match race {
        Race::Human => 9,
        Race::Orc | Race::Dwarf | Race::NightElf => 8,
        Race::Tauren => match gender {
            Gender::Male => 18,
            Gender::Female => 10,
            Gender::None => return false, // Player characters can not be None
        },
        Race::Gnome => 4,
        Race::Undead | Race::Troll => 5,
        Race::Goblin => return false, // Player characters can not be Goblin
    };

    let max_facial_hair = match (race, gender) {
        (Race::Human, Gender::Male) => 8,
        (Race::Human, Gender::Female) => 6,
        (Race::Orc, Gender::Male) => 10,
        (Race::Orc, Gender::Female) => 6,
        (Race::Dwarf, Gender::Male) => 10,
        (Race::Dwarf, Gender::Female) => 5,
        (Race::NightElf, Gender::Male) => 5,
        (Race::NightElf, Gender::Female) => 9,
        (Race::Undead, Gender::Male) => 16,
        (Race::Undead, Gender::Female) => 7,
        (Race::Tauren, Gender::Male) => 6,
        (Race::Tauren, Gender::Female) => 4,
        (Race::Gnome, Gender::Male) => 7,
        (Race::Gnome, Gender::Female) => 6,
        (Race::Troll, Gender::Male) => 10,
        (Race::Troll, Gender::Female) => 5,
        (Race::Goblin, _) | (_, Gender::None) => return false, // Player characters can not be None or Goblin
    };

    let max_face = match (race, gender) {
        (Race::Dwarf, Gender::Male | Gender::Female) => 9,
        (Race::Gnome, Gender::Male | Gender::Female) => 6,
        (Race::Human, Gender::Male) => 11,
        (Race::Human, Gender::Female) => 14,
        (Race::NightElf, Gender::Male | Gender::Female) => 8,
        (Race::Orc, Gender::Male | Gender::Female) => 8,
        (Race::Tauren, Gender::Male) => 4,
        (Race::Tauren, Gender::Female) => 3,
        (Race::Troll, Gender::Male) => 4,
        (Race::Troll, Gender::Female) => 5,
        (Race::Undead, Gender::Male | Gender::Female) => 9,
        (Race::Goblin, _) | (_, Gender::None) => return false, // Player characters can not be Goblin or None
    };

    let max_hairstyle = match (race, gender) {
        (Race::Dwarf, Gender::Male) => 10,
        (Race::Dwarf, Gender::Female) => 13,
        (Race::Gnome, Gender::Male | Gender::Female) => 6,
        (Race::Human, Gender::Male) => 11,
        (Race::Human, Gender::Female) => 18,
        (Race::NightElf, Gender::Male | Gender::Female) => 6,
        (Race::Orc, Gender::Male) => 6,
        (Race::Orc, Gender::Female) => 7,
        (Race::Undead, Gender::Male | Gender::Female) => 9,
        (Race::Tauren, Gender::Male) => 7,
        (Race::Tauren, Gender::Female) => 6,
        (Race::Troll, Gender::Male) => 5,
        (Race::Troll, Gender::Female) => 4,
        (Race::Goblin, _) | (_, Gender::None) => return false, // Player characters can not be Goblin or None
    };

    let max_haircolor = match race {
        Race::Dwarf => 9,
        Race::Gnome => 8,
        Race::Human => 9,
        Race::NightElf => 7,
        Race::Orc => 7,
        Race::Undead => 9,
        Race::Tauren => 2,
        Race::Troll => 9,
        Race::Goblin => return false, // Player characters can not be Goblin
    };

    skin <= max_skin_color
        && facial_hair <= max_facial_hair
        && face <= max_face
        && hair_color <= max_haircolor
        && hair_style <= max_hairstyle
}
