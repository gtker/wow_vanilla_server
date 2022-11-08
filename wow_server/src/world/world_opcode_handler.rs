use crate::world::chat::handle_message;
use crate::world::client::{CharacterScreenProgress, Client};
use crate::world::creature::Creature;
use crate::world::database::WorldDatabase;
use crate::world::world_handler;
use crate::world::world_handler::announce_character_login;
use std::time::SystemTime;
use wow_common::vanilla::position::{get_position_from_str, Position};
use wow_world_messages::vanilla::opcodes::{ClientOpcodeMessage, ServerOpcodeMessage};
use wow_world_messages::vanilla::{
    LogoutResult, LogoutSpeed, MSG_MOVE_FALL_LAND_Server, MSG_MOVE_HEARTBEAT_Server,
    MSG_MOVE_JUMP_Server, MSG_MOVE_SET_FACING_Server, MSG_MOVE_SET_PITCH_Server,
    MSG_MOVE_SET_RUN_MODE_Server, MSG_MOVE_SET_WALK_MODE_Server, MSG_MOVE_START_BACKWARD_Server,
    MSG_MOVE_START_FORWARD_Server, MSG_MOVE_START_PITCH_DOWN_Server,
    MSG_MOVE_START_PITCH_UP_Server, MSG_MOVE_START_STRAFE_LEFT_Server,
    MSG_MOVE_START_STRAFE_RIGHT_Server, MSG_MOVE_START_SWIM_Server,
    MSG_MOVE_START_TURN_LEFT_Server, MSG_MOVE_START_TURN_RIGHT_Server, MSG_MOVE_STOP_PITCH_Server,
    MSG_MOVE_STOP_STRAFE_Server, MSG_MOVE_STOP_SWIM_Server, MSG_MOVE_STOP_Server,
    MSG_MOVE_STOP_TURN_Server, SMSG_LOGOUT_COMPLETE, SMSG_LOGOUT_RESPONSE,
    SMSG_NAME_QUERY_RESPONSE, SMSG_PONG, SMSG_QUERY_TIME_RESPONSE,
};
use wow_world_messages::Guid;

pub async fn handle_received_client_opcodes(
    client: &mut Client,
    clients: &mut [Client],
    creatures: &mut [Creature],
    mut db: WorldDatabase,
    locations: &[(Position, String)],
    move_to_character_screen: &mut Vec<Guid>,
) {
    while let Ok(opcode) = client.received_messages().try_recv() {
        if let Some(info) = opcode.movement_info() {
            client.character_mut().info = info.clone();
        }

        match opcode {
            ClientOpcodeMessage::CMSG_NAME_QUERY(c) => {
                let character = db.get_character_by_guid(c.guid);

                client
                    .send_message(SMSG_NAME_QUERY_RESPONSE {
                        guid: c.guid,
                        character_name: character.name,
                        realm_name: "".to_string(),
                        race: character.race,
                        gender: character.gender,
                        class: character.class,
                    })
                    .await;
            }
            ClientOpcodeMessage::CMSG_WORLD_TELEPORT(c) => {
                let p = Position::new(
                    c.map,
                    c.position.x,
                    c.position.y,
                    c.position.z,
                    c.orientation,
                );
                world_handler::prepare_teleport(p, client).await;
            }
            ClientOpcodeMessage::CMSG_TELEPORT_TO_UNIT(c) => {
                let p = get_position_from_str(&c.name);
                if let Some(p) = p {
                    world_handler::prepare_teleport(p, client).await;
                } else {
                    client
                        .send_system_message(format!("Location not found: '{}'", c.name))
                        .await;
                }
            }
            ClientOpcodeMessage::MSG_MOVE_WORLDPORT_ACK(_) => {
                if !client.in_process_of_teleport {
                    return;
                }
                client.in_process_of_teleport = false;

                for m in world_handler::get_client_login_messages(client.character()) {
                    client.send_opcode(&m).await;
                }

                for c in &mut *clients {
                    announce_character_login(client, c.character()).await;
                }

                for c in &mut *clients {
                    announce_character_login(c, client.character()).await;
                }

                for creature in &mut *creatures {
                    client.send_message(creature.to_message()).await;
                }
            }
            ClientOpcodeMessage::CMSG_MESSAGECHAT(c) => {
                if c.message.starts_with('.') {
                    world_handler::gm_command(
                        client,
                        clients,
                        c.message.trim_start_matches('.'),
                        locations,
                    )
                    .await;

                    return;
                }

                handle_message(client, clients, c).await;
            }
            ClientOpcodeMessage::CMSG_LOGOUT_REQUEST(_) => {
                client
                    .send_message(SMSG_LOGOUT_RESPONSE {
                        result: LogoutResult::Success,
                        speed: LogoutSpeed::Instant,
                    })
                    .await;

                move_to_character_screen.push(client.character().guid);
                client.status = CharacterScreenProgress::CharacterScreen;

                db.replace_character_data(client.character().clone());

                client.send_message(SMSG_LOGOUT_COMPLETE {}).await;
            }
            ClientOpcodeMessage::CMSG_SET_SELECTION(c) => {
                client.character_mut().target = c.target;
            }
            ClientOpcodeMessage::CMSG_QUERY_TIME(_) => {
                client
                    .send_message(SMSG_QUERY_TIME_RESPONSE {
                        time: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as u32,
                    })
                    .await;
            }
            ClientOpcodeMessage::MSG_MOVE_START_FORWARD(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_FORWARD_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_BACKWARD(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_BACKWARD_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP(c) => {
                send_movement_to_clients(
                    MSG_MOVE_STOP_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_STRAFE_LEFT(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_STRAFE_LEFT_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_STRAFE_RIGHT(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_STRAFE_RIGHT_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_STRAFE(c) => {
                send_movement_to_clients(
                    MSG_MOVE_STOP_STRAFE_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_JUMP(c) => {
                send_movement_to_clients(
                    MSG_MOVE_JUMP_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_TURN_LEFT(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_TURN_LEFT_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_TURN_RIGHT(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_TURN_RIGHT_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_TURN(c) => {
                send_movement_to_clients(
                    MSG_MOVE_STOP_TURN_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_PITCH_UP(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_PITCH_UP_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_PITCH_DOWN(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_PITCH_DOWN_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_PITCH(c) => {
                send_movement_to_clients(
                    MSG_MOVE_STOP_PITCH_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_RUN_MODE(c) => {
                send_movement_to_clients(
                    MSG_MOVE_SET_RUN_MODE_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_WALK_MODE(c) => {
                send_movement_to_clients(
                    MSG_MOVE_SET_WALK_MODE_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_FALL_LAND(c) => {
                send_movement_to_clients(
                    MSG_MOVE_FALL_LAND_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_SWIM(c) => {
                send_movement_to_clients(
                    MSG_MOVE_START_SWIM_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_SWIM(c) => {
                send_movement_to_clients(
                    MSG_MOVE_STOP_SWIM_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_FACING(c) => {
                send_movement_to_clients(
                    MSG_MOVE_SET_FACING_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_PITCH(c) => {
                send_movement_to_clients(
                    MSG_MOVE_SET_PITCH_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_HEARTBEAT(c) => {
                send_movement_to_clients(
                    MSG_MOVE_HEARTBEAT_Server {
                        guid: client.character().guid,
                        info: c.info,
                    }
                    .into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::CMSG_MOVE_FALL_RESET(_) => {}
            ClientOpcodeMessage::CMSG_PING(c) => {
                client
                    .send_message(SMSG_PONG {
                        sequence_id: c.sequence_id,
                    })
                    .await;
            }
            ClientOpcodeMessage::CMSG_UPDATE_ACCOUNT_DATA(_) => {
                // Do not spam console, mangos also ignores
            }
            _ => {
                dbg!(opcode);
            }
        }
    }
}

async fn send_movement_to_clients(message: ServerOpcodeMessage, clients: &mut [Client]) {
    for c in clients {
        c.send_opcode(&message).await;
    }
}
