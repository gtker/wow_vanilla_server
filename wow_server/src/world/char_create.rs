use crate::world::character::Character;
use crate::world::database::WorldDatabase;
use wow_common::vanilla::position::get_starting_position;
use wow_common::DEFAULT_RUNNING_SPEED;
use wow_world_messages::vanilla::{Area, MovementInfo, Vector3d, CMSG_CHAR_CREATE};
use wow_world_messages::Guid;

pub(crate) fn create_character(c: CMSG_CHAR_CREATE, db: &WorldDatabase) -> Character {
    let start_zone = get_starting_position(c.race.try_into().unwrap());

    Character {
        guid: db.new_guid().into(),
        name: c.name,
        race: c.race,
        class: c.class,
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
    }
}
