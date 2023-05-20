use crate::world::character::Character;
use crate::world::database::WorldDatabase;
use wow_world_base::vanilla::{PlayerGender, RaceClass};
use wow_world_messages::vanilla::CMSG_CHAR_CREATE;

pub(crate) fn create_character(c: CMSG_CHAR_CREATE, db: &mut WorldDatabase) -> Option<Character> {
    let race_class = RaceClass::try_from((c.race, c.class)).ok()?;
    let gender = PlayerGender::try_from(c.gender).ok()?;

    Some(Character::new(
        db,
        c.name,
        race_class,
        gender,
        c.skin_color,
        c.face,
        c.hair_style,
        c.hair_color,
        c.facial_hair,
    ))
}
