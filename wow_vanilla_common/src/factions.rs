use wow_world_base::vanilla::Race;

pub fn get_race_faction(race: Race) -> i32 {
    match race {
        Race::Human => 1,
        Race::Orc => 2,
        Race::Dwarf => 3,
        Race::NightElf => 4,
        Race::Undead => 5,
        Race::Tauren => 6,
        Race::Gnome => 115,
        Race::Troll => 116,
        Race::Goblin => 1,
    }
}
