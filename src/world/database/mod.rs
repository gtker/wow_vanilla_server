use crate::world::world_opcode_handler::character::Character;
use wow_world_base::vanilla::{PlayerGender, RaceClass};
use wow_world_messages::Guid;

#[derive(Debug, Clone)]
pub struct WorldDatabase {
    characters_for_all_accounts: Vec<Character>,
    next_guid: u64,
}

impl WorldDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            characters_for_all_accounts: vec![],
            next_guid: 0,
        };

        let c = Character::test_character(
            &mut db,
            "Dev",
            RaceClass::HumanWarrior,
            PlayerGender::Female,
        );
        db.create_character_in_account("", c);
        let c = Character::test_character(
            &mut db,
            "HumOne",
            RaceClass::HumanWarrior,
            PlayerGender::Female,
        );
        db.create_character_in_account("", c);
        let c = Character::test_character(
            &mut db,
            "HumTwo",
            RaceClass::HumanWarrior,
            PlayerGender::Male,
        );
        db.create_character_in_account("", c);

        db
    }

    pub fn get_characters_for_account(&self, _account_name: &str) -> Vec<Character> {
        self.characters_for_all_accounts.clone()
    }

    pub fn create_character_in_account(&mut self, _account_name: &str, character: Character) {
        self.characters_for_all_accounts.push(character);
    }

    pub fn new_guid(&mut self) -> u64 {
        let g = self.next_guid;
        self.next_guid += 1;
        g
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
