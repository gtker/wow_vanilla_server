use crate::world::character::Character;
use crate::world::database::WorldDatabase;
use wow_world_base::wrath::RaceClass;
use wow_world_base::DEFAULT_RUNNING_SPEED;
use wow_world_messages::wrath::{Area, MovementInfo, Vector3d, CMSG_CHAR_CREATE};
use wow_world_messages::Guid;

pub(crate) fn create_character(c: CMSG_CHAR_CREATE, db: &WorldDatabase) -> Option<Character> {
    let race_class = RaceClass::try_from((c.race, c.class)).ok()?;
    let player_race = race_class.to_race_class().0;
    let start_zone = player_race.wrath_starting_position(c.class);

    Some(Character {
        guid: db.new_guid().into(),
        name: c.name,
        race_class,
        gender: c.gender,
        skin: c.skin_color,
        face: c.face,
        hairstyle: c.hair_style,
        haircolor: c.hair_color,
        facialhair: c.facial_hair,
        level: 1,
        area: Area::NorthshireAbbey,
        map: start_zone.map,
        info: MovementInfo {
            flags: Default::default(),
            extra_flags: Default::default(),
            timestamp: 0,
            position: Vector3d {
                x: start_zone.x,
                y: start_zone.y,
                z: start_zone.z,
            },
            orientation: start_zone.orientation,
            fall_time: 0.0,
        },
        movement_speed: DEFAULT_RUNNING_SPEED,
        target: Guid::new(0),
        attacking: false,
        auto_attack_timer: 0.0,
    })
}
