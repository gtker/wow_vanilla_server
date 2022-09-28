use crate::world::char_create;
use crate::world::client::{CharacterScreenProgress, Client};
use crate::world::database::WorldDatabase;
use crate::world::world_handler::get_client_login_messages;
use wow_vanilla_common::race::{character_features_are_valid, character_race_class_combo_is_valid};
use wow_world_messages::vanilla::opcodes::ClientOpcodeMessage;
use wow_world_messages::vanilla::{
    Character, WorldResult, SMSG_CHAR_CREATE, SMSG_CHAR_ENUM, SMSG_PONG,
};

pub async fn handle_character_screen_opcodes(client: &mut Client, mut db: WorldDatabase) {
    while let Ok(opcode) = client.received_messages().try_recv() {
        match opcode {
            ClientOpcodeMessage::CMSG_PING(c) => {
                client
                    .send_message(SMSG_PONG {
                        sequence_id: c.sequence_id,
                    })
                    .await;
            }
            ClientOpcodeMessage::CMSG_CHAR_ENUM(_) => {
                let characters: Vec<Character> = db
                    .get_characters_for_account(&client.account_name())
                    .into_iter()
                    .map(|a| a.into())
                    .collect();

                client.send_message(SMSG_CHAR_ENUM { characters }).await;
            }
            ClientOpcodeMessage::CMSG_CHAR_CREATE(c) => {
                let character = char_create::create_character(c, &db);

                if character_features_are_valid(
                    character.race,
                    character.gender,
                    character.skin,
                    character.facialhair,
                    character.face,
                    character.haircolor,
                    character.hairstyle,
                ) && character_race_class_combo_is_valid(character.race, character.class)
                {
                    db.create_character_in_account(client.account_name(), character);

                    client
                        .send_message(SMSG_CHAR_CREATE {
                            result: WorldResult::CharCreateSuccess,
                        })
                        .await;
                } else {
                    client
                        .send_message(SMSG_CHAR_CREATE {
                            result: WorldResult::CharCreateError,
                        })
                        .await;
                }
            }
            ClientOpcodeMessage::CMSG_CHAR_DELETE(c) => {
                db.delete_character_by_guid(client.account_name(), c.guid);
            }
            ClientOpcodeMessage::CMSG_PLAYER_LOGIN(c) => {
                let character = db.get_character_by_guid(c.guid);

                *client.character_mut() = character;
                client.status = CharacterScreenProgress::WaitingToLogIn;

                for m in get_client_login_messages(client.character()) {
                    client.send_opcode(&m).await;
                }
            }
            e => {
                dbg!(e);
            }
        }
    }
}
