use crate::world::character::Character;
use std::io::ErrorKind;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use wow_srp::vanilla_header::{EncrypterHalf, HeaderCrypto};
use wow_world_base::geometry::distance_between;
use wow_world_base::vanilla::position::Position;
use wow_world_messages::errors::{ExpectedOpcodeError, ParseError};
use wow_world_messages::vanilla::opcodes::{ClientOpcodeMessage, ServerOpcodeMessage};
use wow_world_messages::vanilla::{
    Language, MovementInfo, PlayerChatTag, SMSG_MESSAGECHAT_ChatType, ServerMessage, Vector3d,
    SMSG_MESSAGECHAT,
};
use wow_world_messages::Guid;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum CharacterScreenProgress {
    CharacterScreen,
    WaitingToLogIn(Guid),
}

#[derive(Debug)]
pub struct Client {
    character: Character,
    pub in_process_of_teleport: bool,
    pub location_index: usize,
    pub status: CharacterScreenProgress,
    received_messages: Receiver<ClientOpcodeMessage>,
    write: OwnedWriteHalf,
    encrypter: EncrypterHalf,
    account_name: String,
    pub reader_handle: JoinHandle<()>,
}

impl Client {
    pub(crate) fn into_character_screen_client(self) -> CharacterScreenClient {
        CharacterScreenClient {
            in_process_of_teleport: self.in_process_of_teleport,
            location_index: self.location_index,
            status: self.status,
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
        m.tokio_write_encrypted_server(&mut self.write, &mut self.encrypter)
            .await
            .unwrap();
    }

    pub async fn send_opcode(&mut self, m: &ServerOpcodeMessage) {
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

    pub fn coordinates(&self) -> (f32, f32, f32) {
        (
            self.character().info.position.x,
            self.character().info.position.y,
            self.character().info.position.z,
        )
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
        let (self_x, self_y, self_z) = self.coordinates();
        let (other_x, other_y, other_z) = other.coordinates();

        if self.character().map == other.character().map {
            Some(distance_between(
                Vector3d {
                    x: self_x,
                    y: self_y,
                    z: self_z,
                },
                Vector3d {
                    x: other_x,
                    y: other_y,
                    z: other_z,
                },
            ))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct CharacterScreenClient {
    pub in_process_of_teleport: bool,
    pub location_index: usize,
    pub status: CharacterScreenProgress,
    received_messages: Receiver<ClientOpcodeMessage>,
    write: OwnedWriteHalf,
    encrypter: EncrypterHalf,
    account_name: String,
    pub reader_handle: JoinHandle<()>,
}

impl CharacterScreenClient {
    pub fn into_client(self, character: Character) -> Client {
        Client {
            character,
            in_process_of_teleport: self.in_process_of_teleport,
            location_index: self.location_index,
            status: self.status,
            received_messages: self.received_messages,
            write: self.write,
            encrypter: self.encrypter,
            account_name: self.account_name,
            reader_handle: self.reader_handle,
        }
    }

    pub fn new(account_name: String, stream: TcpStream, encryption: HeaderCrypto) -> Self {
        let (read, write) = stream.into_split();
        let (encrypter, decrypter) = encryption.split();

        let (client_send, client_recv) = mpsc::channel(32);

        let reader_handle = tokio::spawn(async move {
            let mut read = read;
            let mut decrypter = decrypter;
            loop {
                let msg =
                    ClientOpcodeMessage::tokio_read_encrypted(&mut read, &mut decrypter).await;
                let msg = match msg {
                    Ok(m) => m,
                    Err(e) => {
                        match e {
                            ExpectedOpcodeError::Opcode { opcode, size, name } => {
                                let mut v = vec![0_u8; size as usize];
                                read.read_exact(&mut v).await.unwrap();
                                dbg!(name, opcode, size, v);
                            }
                            ExpectedOpcodeError::Parse(ref p) => match p {
                                ParseError::Io(i) => match i.kind() {
                                    ErrorKind::UnexpectedEof => {
                                        break;
                                    }
                                    _ => println!("DC: {:#?}", e),
                                },
                                _ => println!("DC: {:#?}", e),
                            },
                        }
                        continue;
                    }
                };

                client_send.send(msg).await.unwrap();
            }
        });

        Self {
            in_process_of_teleport: false,
            location_index: 0,
            status: CharacterScreenProgress::CharacterScreen,
            received_messages: client_recv,
            write,
            encrypter,
            account_name,
            reader_handle,
        }
    }

    pub fn account_name(&self) -> &str {
        &self.account_name
    }

    pub fn received_messages(&mut self) -> &mut Receiver<ClientOpcodeMessage> {
        &mut self.received_messages
    }

    pub async fn send_message(&mut self, m: impl ServerMessage + Sync) {
        m.tokio_write_encrypted_server(&mut self.write, &mut self.encrypter)
            .await
            .unwrap();
    }

    pub async fn send_opcode(&mut self, m: &ServerOpcodeMessage) {
        m.tokio_write_encrypted_server(&mut self.write, &mut self.encrypter)
            .await
            .unwrap();
    }
}
