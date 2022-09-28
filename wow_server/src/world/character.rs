use wow_vanilla_common::base_stats::{calculate_health, calculate_mana, get_base_stats, BaseStats};
use wow_vanilla_common::class::get_power_for_class;
use wow_vanilla_common::{Class, Map, Race};
use wow_world_messages::vanilla::{
    Area, CharacterFlags, CharacterGear, Gender, MovementInfo, Power,
};
use wow_world_messages::Guid;

#[derive(Debug, Clone)]
pub struct Character {
    pub guid: Guid,
    pub name: String,
    pub race: Race,
    pub class: Class,
    pub gender: Gender,
    pub skin: u8,
    pub face: u8,
    pub hairstyle: u8,
    pub haircolor: u8,
    pub facialhair: u8,
    pub level: u8,
    pub area: Area,
    pub map: Map,
    pub info: MovementInfo,
    pub movement_speed: f32,
    pub target: Guid,
}

impl Character {
    fn default_stats(&self) -> BaseStats {
        get_base_stats(self.race, self.class, self.level)
    }

    pub fn strength(&self) -> i32 {
        self.default_stats().strength.into()
    }

    pub fn base_health(&self) -> i32 {
        self.default_stats().health.into()
    }

    pub fn max_health(&self) -> i32 {
        calculate_health(self.default_stats().health, self.default_stats().stamina).into()
    }

    pub fn base_mana(&self) -> i32 {
        self.default_stats().mana.into()
    }

    pub fn max_mana(&self) -> i32 {
        if get_power_for_class(self.class) == Power::Mana {
            calculate_mana(self.default_stats().mana, self.default_stats().intellect).into()
        } else {
            0
        }
    }

    pub fn agility(&self) -> i32 {
        self.default_stats().agility.into()
    }

    pub fn stamina(&self) -> i32 {
        self.default_stats().stamina.into()
    }

    pub fn intellect(&self) -> i32 {
        self.default_stats().intellect.into()
    }

    pub fn spirit(&self) -> i32 {
        self.default_stats().spirit.into()
    }
}

impl From<Character> for wow_world_messages::vanilla::Character {
    fn from(e: Character) -> Self {
        wow_world_messages::vanilla::Character {
            guid: e.guid,
            name: e.name,
            race: e.race,
            class: e.class,
            gender: e.gender,
            skin: e.skin,
            face: e.face,
            hair_style: e.hairstyle,
            hair_color: e.haircolor,
            facial_hair: e.facialhair,
            level: e.level,
            area: e.area,
            map: e.map,
            position: e.info.position,
            guild_id: 0,
            flags: CharacterFlags::empty(),
            first_login: 0,
            pet_display_id: 0,
            pet_level: 0,
            pet_family: 0,
            equipment: [CharacterGear::default(); 19],
        }
    }
}

impl Eq for Character {}

impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        self.guid == other.guid
    }
}
