use crate::world::database::WorldDatabase;
use crate::world::item::Item;
use crate::world::DESIRED_TIMESTEP;
use wow_items::vanilla::lookup_item;
use wow_world_base::vanilla::{Level, Map, PlayerGender, RaceClass, Vector3d};
use wow_world_base::{calculate_health, calculate_mana};
use wow_world_base::{BaseStats, DEFAULT_RUNNING_SPEED};
use wow_world_messages::vanilla::{Area, CharacterGear, CreatureFamily, MovementInfo, Power};
use wow_world_messages::Guid;

#[derive(Debug, Clone)]
pub struct Character {
    pub guid: Guid,
    pub name: String,
    pub race_class: RaceClass,
    pub gender: PlayerGender,
    pub skin: u8,
    pub face: u8,
    pub hairstyle: u8,
    pub haircolor: u8,
    pub facialhair: u8,
    pub level: Level,
    pub area: Area,
    pub map: Map,
    pub info: MovementInfo,
    pub movement_speed: f32,
    pub target: Guid,
    pub attacking: bool,
    pub auto_attack_timer: f32,
    pub items: Vec<Item>,
}

impl Character {
    fn default_stats(&self) -> BaseStats {
        self.race_class
            .base_stats_for(self.level.as_int())
            .unwrap_or(self.race_class.base_stats()[0])
    }

    pub fn test_character(
        db: &mut WorldDatabase,
        name: impl Into<String>,
        race_class: RaceClass,
        gender: PlayerGender,
    ) -> Self {
        let mut c = Self::new(db, name, race_class, gender, 0, 0, 0, 0, 0);
        c.level = Level::new_vanilla_max_level_player();
        c
    }

    pub fn new(
        db: &mut WorldDatabase,
        name: impl Into<String>,
        race_class: RaceClass,
        gender: PlayerGender,
        skin: u8,
        face: u8,
        hair_style: u8,
        hair_color: u8,
        facial_hair: u8,
    ) -> Self {
        let start = race_class.starting_position();

        let items = {
            race_class
                .starter_items()
                .iter()
                .map(|item| Item::new(lookup_item(item.item).unwrap(), db))
        }
        .collect();

        Self {
            guid: db.new_guid().into(),
            name: name.into(),
            race_class,
            gender,
            skin,
            face,
            hairstyle: hair_style,
            haircolor: hair_color,
            facialhair: facial_hair,
            level: Level::new_player(),
            area: Default::default(),
            map: start.map,
            info: MovementInfo {
                flags: Default::default(),
                timestamp: 0,
                position: Vector3d {
                    x: start.x,
                    y: start.y,
                    z: start.z,
                },
                orientation: start.orientation,
                fall_time: 0.0,
            },
            movement_speed: DEFAULT_RUNNING_SPEED,
            target: Default::default(),
            attacking: false,
            auto_attack_timer: 0.0,
            items,
        }
    }

    pub fn update_auto_attack_timer(&mut self) {
        if self.auto_attack_timer > 0.0 {
            self.auto_attack_timer -= DESIRED_TIMESTEP;
        }
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
        if self.race_class.class().power_type() == Power::Mana {
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
            race: e.race_class.race().into(),
            class: e.race_class.class(),
            gender: e.gender.into(),
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
            flags: Default::default(),
            first_login: false,
            pet_display_id: 0,
            pet_level: Level::zero(),
            pet_family: CreatureFamily::None,
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
