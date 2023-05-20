mod char_create;
mod character;
mod character_screen_handler;
pub mod chat;
mod client;
mod creature;
mod database;
pub(crate) mod item;
mod world_handler;
pub mod world_opcode_handler;

use crate::world::client::CharacterScreenClient;
use crate::world::database::WorldDatabase;
use crate::world::world_handler::World;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use wow_srp::normalized_string::NormalizedString;
use wow_srp::server::SrpServer;
use wow_srp::vanilla_header::ProofSeed;
use wow_world_messages::vanilla::tokio_expect_client_message;
use wow_world_messages::vanilla::*;

pub async fn world(users: Arc<Mutex<HashMap<String, SrpServer>>>) {
    let listener = TcpListener::bind("0.0.0.0:8085").await.unwrap();
    let (tx, rx) = mpsc::channel(32);
    let world = World::new(rx);

    tokio::spawn(run_world(world));

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        tokio::spawn(character_screen(stream, users.clone(), tx.clone()));
    }
}

pub const DESIRED_TIMESTEP: f32 = 1.0 / 10.0;

async fn run_world(mut world: World) {
    let mut db = WorldDatabase::new();

    loop {
        let before = Instant::now();

        world.tick(&mut db).await;

        let after = Instant::now();

        let tick_duration = after.duration_since(before);

        if tick_duration.as_secs_f32() < DESIRED_TIMESTEP {
            sleep(Duration::from_secs_f32(
                DESIRED_TIMESTEP - tick_duration.as_secs_f32(),
            ))
            .await;
        } else {
            println!("Timestep took too long: '{}'", tick_duration.as_secs_f32());
        }
    }
}

async fn character_screen(
    mut stream: TcpStream,
    users: Arc<Mutex<HashMap<String, SrpServer>>>,
    world: Sender<CharacterScreenClient>,
) {
    let seed = ProofSeed::new();

    SMSG_AUTH_CHALLENGE {
        server_seed: seed.seed(),
    }
    .tokio_write_unencrypted_server(&mut stream)
    .await
    .unwrap();

    let c = tokio_expect_client_message::<CMSG_AUTH_SESSION, _>(&mut stream)
        .await
        .unwrap();
    let account_name = c.username;

    let session_key = {
        let mut server = users.lock().unwrap();
        *server.get_mut(&account_name).unwrap().session_key()
    };

    let mut encryption = seed
        .into_header_crypto(
            &NormalizedString::new(&account_name).unwrap(),
            session_key,
            c.client_proof,
            c.client_seed,
        )
        .unwrap();

    SMSG_AUTH_RESPONSE {
        result: SMSG_AUTH_RESPONSE_WorldResult::AuthOk {
            billing_flags: 0,
            billing_rested: 0,
            billing_time: 0,
        },
    }
    .tokio_write_encrypted_server(&mut stream, encryption.encrypter())
    .await
    .unwrap();

    world
        .send(CharacterScreenClient::new(account_name, stream, encryption))
        .await
        .unwrap();
}
