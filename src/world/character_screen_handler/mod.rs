use crate::world::client::{CharacterScreenClient, CharacterScreenProgress};
use crate::world::database::WorldDatabase;
use crate::world::world_handler::get_client_login_messages;
use crate::world::world_opcode_handler::write_client_test;
use wow_world_messages::vanilla::opcodes::ClientOpcodeMessage;
use wow_world_messages::vanilla::{
    Character, WorldResult, SMSG_CHAR_CREATE, SMSG_CHAR_ENUM, SMSG_PONG,
};

mod char_create;

pub async fn handle_character_screen_opcodes(
    client: &mut CharacterScreenClient,
    db: &mut WorldDatabase,
) {
    while let Ok(opcode) = client.received_messages().try_recv() {
        match opcode {
            ClientOpcodeMessage::CMSG_PING(c) => {
                client
                    .send_message(SMSG_PONG {
                        sequence_id: c.sequence_id,
                    })
                    .await;
            }
            ClientOpcodeMessage::CMSG_CHAR_ENUM => {
                let characters: Vec<Character> = db
                    .get_characters_for_account(&client.account_name())
                    .into_iter()
                    .map(|a| a.into())
                    .collect();

                client.send_message(SMSG_CHAR_ENUM { characters }).await;
            }
            ClientOpcodeMessage::CMSG_CHAR_CREATE(c) => {
                let character = char_create::create_character(c, db);

                if let Some(character) = character {
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
                client.status = CharacterScreenProgress::WaitingToLogIn(c.guid);

                let character = db.get_character_by_guid(c.guid);

                for m in get_client_login_messages(&character) {
                    client.send_opcode(&m).await;
                }
            }
            e => {
                dbg!(&e);
                write_client_test(&e);
            }
        }
    }
}
