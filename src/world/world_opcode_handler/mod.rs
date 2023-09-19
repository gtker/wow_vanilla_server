use crate::file_utils::append_string_to_file;
use crate::world::database::WorldDatabase;
use crate::world::world::client::Client;
use crate::world::world::pathfinding_maps::PathfindingMaps;
use crate::world::world_opcode_handler::entities::Entities;
use crate::world::world_opcode_handler::opcode_handler::handle_opcodes;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use wow_world_messages::vanilla::opcodes::{ClientOpcodeMessage, ServerOpcodeMessage};
use wow_world_messages::vanilla::ServerMessage;

pub mod character;
pub mod chat;
pub mod creature;
pub(crate) mod entities;
pub(crate) mod gm_command;
pub mod inventory;
pub(crate) mod item;
mod opcode_handler;

pub(crate) async fn handle_received_client_opcodes(
    client: &mut Client,
    entities: &mut Entities<'_>,
    db: &mut WorldDatabase,
    move_to_character_screen: &mut bool,
    maps: &mut PathfindingMaps,
) {
    while let Ok(opcode) = client.received_messages().try_recv() {
        handle_opcodes(client, entities, db, move_to_character_screen, opcode, maps).await;
    }
}

async fn send_to_all(
    message: impl ServerMessage + Clone + Sync,
    client: &mut Client,
    clients: &mut [Client],
) {
    for client in clients {
        client.send_message(message.clone()).await;
    }

    client.send_message(message).await;
}

async fn send_movement_to_clients(message: ServerOpcodeMessage, clients: &mut [Client]) {
    for c in clients {
        c.send_opcode(&message).await;
    }
}

pub(crate) fn write_client_test(msg: &ClientOpcodeMessage) {
    if let Some(contents) = msg.to_test_case_string() {
        write_test_case_inner(contents.as_str(), msg.message_name());
    } else {
        dbg!(&msg);
    }
}

pub(crate) fn write_server_test(msg: &ServerOpcodeMessage) {
    if let Some(contents) = msg.to_test_case_string() {
        write_test_case_inner(contents.as_str(), msg.message_name());
    }
}

pub(crate) fn write_test_case_inner(contents: &str, message_name: &str) {
    if let Some(path) = find_wowm_file(message_name) {
        println!("Added {message_name} to {path}", path = path.display());
        append_string_to_file("\n", &path);
        append_string_to_file(&contents, &path);
    } else {
        let path = Path::new("./tests.wowm");
        println!("Added {message_name} to {path}", path = path.display());
        append_string_to_file("\n", path);
        append_string_to_file(&contents, path);
    }
}

fn find_wowm_file(name: &str) -> Option<PathBuf> {
    let search_name = format!(" {name} ");

    for file in WalkDir::new(Path::new("../wow_messages/wow_message_parser/wowm"))
        .into_iter()
        .filter_map(|a| a.ok())
    {
        let Ok(contents) = read_to_string(file.path()) else {
            continue;
        };

        if contents.contains(&search_name) {
            return Some(file.path().to_path_buf());
        }
    }

    None
}
