pub(crate) mod character_screen_client;

use crate::world::world_opcode_handler::character::Character;
use crate::world::world_opcode_handler::{write_server_test, write_test_case_inner};
use character_screen_client::{CharacterScreenClient, CharacterScreenProgress};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use wow_srp::vanilla_header::EncrypterHalf;
use wow_world_base::geometry::distance_between;
use wow_world_base::vanilla::position::Position;
use wow_world_messages::vanilla::opcodes::{ClientOpcodeMessage, ServerOpcodeMessage};
use wow_world_messages::vanilla::{
    Language, MovementInfo, PlayerChatTag, SMSG_MESSAGECHAT_ChatType, ServerMessage, Vector3d,
    SMSG_MESSAGECHAT,
};
use wow_world_messages::Guid;

#[derive(Debug)]
pub struct Client {
    character: Character,
    pub in_process_of_teleport: bool,
    received_messages: Receiver<ClientOpcodeMessage>,
    write: OwnedWriteHalf,
    encrypter: EncrypterHalf,
    account_name: String,
    pub reader_handle: JoinHandle<()>,
}

impl Client {
    pub(crate) fn into_character_screen_client(self) -> CharacterScreenClient {
        CharacterScreenClient {
            status: CharacterScreenProgress::CharacterScreen,
            received_messages: self.received_messages,
            write: self.write,
            encrypter: self.encrypter,
            account_name: self.account_name,
            reader_handle: self.reader_handle,
        }
    }

    pub fn character(&self) -> &Character {
        &self.character
    }

    pub fn character_mut(&mut self) -> &mut Character {
        &mut self.character
    }

    pub fn set_movement_info(&mut self, info: MovementInfo) {
        self.character.info = info;
    }

    pub fn received_messages(&mut self) -> &mut Receiver<ClientOpcodeMessage> {
        &mut self.received_messages
    }

    pub async fn send_message(&mut self, m: impl ServerMessage + Sync) {
        if let Some(contents) = m.to_test_case_string() {
            write_test_case_inner(&contents, m.message_name());
        }

        m.tokio_write_encrypted_server(&mut self.write, &mut self.encrypter)
            .await
            .unwrap();
    }

    pub async fn send_opcode(&mut self, m: &ServerOpcodeMessage) {
        write_server_test(m);

        m.tokio_write_encrypted_server(&mut self.write, &mut self.encrypter)
            .await
            .unwrap();
    }

    pub async fn send_system_message(&mut self, s: impl Into<String>) {
        self.send_message(SMSG_MESSAGECHAT {
            chat_type: SMSG_MESSAGECHAT_ChatType::System {
                sender2: Guid::new(0),
            },
            language: Language::Universal,
            message: s.into(),
            tag: PlayerChatTag::None,
        })
        .await;
    }

    pub fn position(&self) -> Position {
        Position::new(
            self.character().map,
            self.character().info.position.x,
            self.character().info.position.y,
            self.character().info.position.z,
            self.character().info.orientation,
        )
    }

    pub fn distance_to_center(&self, other: &Self) -> Option<f32> {
        let position = other.position();
        self.distance_to_position(&position)
    }

    pub fn distance_to_position(&self, position: &Position) -> Option<f32> {
        let self_vector = self.position();
        let self_vector = Vector3d {
            x: self_vector.x,
            y: self_vector.y,
            z: self_vector.z,
        };
        let position_vector = Vector3d {
            x: position.x,
            y: position.y,
            z: position.z,
        };

        if self.character().map == position.map {
            Some(distance_between(self_vector, position_vector))
        } else {
            None
        }
    }
}
