use crate::world::world::client::Client;
use crate::world::world_opcode_handler::character::Character;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use wow_srp::vanilla_header::{EncrypterHalf, HeaderCrypto};
use wow_world_base::shared::Guid;
use wow_world_messages::errors::ExpectedOpcodeError;
use wow_world_messages::vanilla::opcodes::{ClientOpcodeMessage, ServerOpcodeMessage};
use wow_world_messages::vanilla::ServerMessage;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum CharacterScreenProgress {
    CharacterScreen,
    WaitingToLogIn(Guid),
}

#[derive(Debug)]
pub struct CharacterScreenClient {
    pub status: CharacterScreenProgress,
    pub(super) received_messages: Receiver<ClientOpcodeMessage>,
    pub(super) write: OwnedWriteHalf,
    pub(super) encrypter: EncrypterHalf,
    pub(super) account_name: String,
    pub reader_handle: JoinHandle<()>,
}

impl CharacterScreenClient {
    pub fn into_client(self, character: Character) -> Client {
        Client {
            character,
            in_process_of_teleport: false,
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
                            ExpectedOpcodeError::Parse(ref p) => {
                                println!("{:#?}", p);
                            }
                            ExpectedOpcodeError::Io(_) => {
                                break;
                            }
                        }
                        continue;
                    }
                };

                client_send.send(msg).await.unwrap();
            }
        });

        Self {
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
