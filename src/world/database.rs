use crate::world::character::Character;
use wow_world_base::wrath::{PlayerGender, RaceClass};
use wow_world_messages::Guid;

#[derive(Debug, Clone)]
pub struct WorldDatabase {
    characters_for_all_accounts: Vec<Character>,
}

impl WorldDatabase {
    pub fn new() -> Self {
        Self {
            characters_for_all_accounts: vec![
                Character::test_character(
                    Guid::new(4),
                    "Dev",
                    RaceClass::HumanWarrior,
                    PlayerGender::Female,
                ),
                Character::test_character(
                    Guid::new(5),
                    "HumOne",
                    RaceClass::HumanWarrior,
                    PlayerGender::Female,
                ),
                Character::test_character(
                    Guid::new(6),
                    "HumTwo",
                    RaceClass::HumanWarrior,
                    PlayerGender::Male,
                ),
            ],
        }
    }

    pub fn get_characters_for_account(&self, _account_name: &str) -> Vec<Character> {
        self.characters_for_all_accounts.clone()
    }

    pub fn create_character_in_account(&mut self, _account_name: &str, character: Character) {
        self.characters_for_all_accounts.push(character);
    }

    pub fn new_guid(&self) -> u64 {
        self.characters_for_all_accounts.last().unwrap().guid.guid() + 1
    }

    pub fn get_character_by_guid(&self, guid: Guid) -> Character {
        self.characters_for_all_accounts
            .iter()
            .find(|a| a.guid == guid)
            .unwrap()
            .clone()
    }

    pub fn replace_character_data(&mut self, c: Character) {
        let guid = c.guid;
        *self
            .characters_for_all_accounts
            .iter_mut()
            .find(|a| a.guid == guid)
            .unwrap() = c;
    }

    pub fn delete_character_by_guid(&mut self, _username: &str, guid: Guid) {
        let index = self
            .characters_for_all_accounts
            .iter()
            .enumerate()
            .find(|a| a.1.guid == guid)
            .unwrap()
            .0;
        self.characters_for_all_accounts.remove(index);
    }
}
