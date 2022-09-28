use crate::world::character::Character;
use std::sync::Arc;
use std::sync::Mutex;
use wow_vanilla_common::position::{get_position, PositionIdentifier};
use wow_vanilla_common::{Class, Race, DEFAULT_RUNNING_SPEED};
use wow_world_messages::vanilla::{Area, Gender, MovementInfo, Vector3d};
use wow_world_messages::Guid;

#[derive(Debug, Clone)]
pub struct WorldDatabase {
    characters_for_all_accounts: Arc<Mutex<Vec<Character>>>,
}

impl WorldDatabase {
    pub fn new() -> Self {
        let dev = get_position(PositionIdentifier::DesignerIsland);
        let human = get_position(PositionIdentifier::HumanStartZone);

        Self {
            characters_for_all_accounts: Arc::new(Mutex::new(vec![
                Character {
                    guid: Guid::new(4),
                    name: "Dev".to_string(),
                    race: Race::Human,
                    class: Class::Warrior,
                    gender: Gender::Female,
                    skin: 0,
                    face: 0,
                    hairstyle: 0,
                    haircolor: 0,
                    facialhair: 0,
                    level: 1,
                    area: Area::DesignerIsland,
                    map: dev.map,
                    info: MovementInfo {
                        flags: Default::default(),
                        timestamp: 0,
                        position: Vector3d {
                            x: dev.x,
                            y: dev.y,
                            z: dev.z,
                        },
                        orientation: dev.orientation,
                        fall_time: 0.0,
                    },
                    movement_speed: DEFAULT_RUNNING_SPEED,
                    target: Guid::new(0),
                },
                Character {
                    guid: Guid::new(5),
                    name: "HumOne".to_string(),
                    race: Race::Human,
                    class: Class::Warrior,
                    gender: Gender::Female,
                    skin: 0,
                    face: 0,
                    hairstyle: 0,
                    haircolor: 0,
                    facialhair: 0,
                    level: 1,
                    area: Default::default(),
                    map: human.map,
                    info: MovementInfo {
                        flags: Default::default(),
                        timestamp: 0,
                        position: Vector3d {
                            x: human.x,
                            y: human.y,
                            z: human.z,
                        },
                        orientation: human.orientation,
                        fall_time: 0.0,
                    },
                    movement_speed: DEFAULT_RUNNING_SPEED,
                    target: Guid::new(0),
                },
                Character {
                    guid: Guid::new(6),
                    name: "HumTwo".to_string(),
                    race: Race::Human,
                    class: Class::Warrior,
                    gender: Gender::Male,
                    skin: 0,
                    face: 0,
                    hairstyle: 0,
                    haircolor: 0,
                    facialhair: 0,
                    level: 1,
                    area: Default::default(),
                    map: human.map,
                    info: MovementInfo {
                        flags: Default::default(),
                        timestamp: 0,
                        position: Vector3d {
                            x: human.x,
                            y: human.y,
                            z: human.z,
                        },
                        orientation: human.orientation,
                        fall_time: 0.0,
                    },
                    movement_speed: DEFAULT_RUNNING_SPEED,
                    target: Guid::new(0),
                },
            ])),
        }
    }

    pub fn get_characters_for_account(&self, _account_name: &str) -> Vec<Character> {
        self.characters_for_all_accounts.lock().unwrap().clone()
    }

    pub fn create_character_in_account(&mut self, _account_name: &str, character: Character) {
        self.characters_for_all_accounts
            .lock()
            .unwrap()
            .push(character);
    }

    pub fn new_guid(&self) -> u64 {
        self.characters_for_all_accounts
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .guid
            .guid()
            + 1
    }

    pub fn get_character_by_guid(&self, guid: Guid) -> Character {
        self.characters_for_all_accounts
            .lock()
            .unwrap()
            .iter()
            .find(|a| a.guid == guid)
            .unwrap()
            .clone()
    }

    pub fn replace_character_data(&mut self, c: Character) {
        let guid = c.guid;
        *self
            .characters_for_all_accounts
            .lock()
            .unwrap()
            .iter_mut()
            .find(|a| a.guid == guid)
            .unwrap() = c;
    }

    pub fn delete_character_by_guid(&mut self, _username: &str, guid: Guid) {
        let index = self
            .characters_for_all_accounts
            .lock()
            .unwrap()
            .iter()
            .enumerate()
            .find(|a| a.1.guid == guid)
            .unwrap()
            .0;
        self.characters_for_all_accounts
            .lock()
            .unwrap()
            .remove(index);
    }
}
