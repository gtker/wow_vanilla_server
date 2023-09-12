use crate::file_utils::append_string_to_file;
use crate::world::chat::handle_message;
use crate::world::client::{CharacterScreenProgress, Client};
use crate::world::creature::Creature;
use crate::world::database::WorldDatabase;
use crate::world::world_handler;
use crate::world::world_handler::{announce_character_login, gm_command};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;
use wow_world_base::combat::UNARMED_SPEED;
use wow_world_base::vanilla::position::{position_from_str, Position};
use wow_world_base::vanilla::trigger::Trigger;
use wow_world_base::vanilla::{CreatureFamily, Guid, HitInfo};
use wow_world_messages::vanilla::opcodes::{ClientOpcodeMessage, ServerOpcodeMessage};
use wow_world_messages::vanilla::{
    item_to_name_query_response, item_to_query_response, DamageInfo, LogoutResult, LogoutSpeed,
    MSG_MOVE_FALL_LAND_Server, MSG_MOVE_HEARTBEAT_Server, MSG_MOVE_JUMP_Server,
    MSG_MOVE_SET_FACING_Server, MSG_MOVE_SET_PITCH_Server, MSG_MOVE_SET_RUN_MODE_Server,
    MSG_MOVE_SET_WALK_MODE_Server, MSG_MOVE_START_BACKWARD_Server, MSG_MOVE_START_FORWARD_Server,
    MSG_MOVE_START_PITCH_DOWN_Server, MSG_MOVE_START_PITCH_UP_Server,
    MSG_MOVE_START_STRAFE_LEFT_Server, MSG_MOVE_START_STRAFE_RIGHT_Server,
    MSG_MOVE_START_SWIM_Server, MSG_MOVE_START_TURN_LEFT_Server, MSG_MOVE_START_TURN_RIGHT_Server,
    MSG_MOVE_STOP_PITCH_Server, MSG_MOVE_STOP_STRAFE_Server, MSG_MOVE_STOP_SWIM_Server,
    MSG_MOVE_STOP_Server, MSG_MOVE_STOP_TURN_Server, Object, Object_UpdateType,
    SMSG_CREATURE_QUERY_RESPONSE_found, ServerMessage, UpdateMask, UpdatePlayerBuilder,
    VisibleItem, VisibleItemIndex, SMSG_ATTACKERSTATEUPDATE, SMSG_ATTACKSTART, SMSG_ATTACKSTOP,
    SMSG_CREATURE_QUERY_RESPONSE, SMSG_EMOTE, SMSG_ITEM_QUERY_SINGLE_RESPONSE,
    SMSG_LOGOUT_COMPLETE, SMSG_LOGOUT_RESPONSE, SMSG_NAME_QUERY_RESPONSE, SMSG_PONG,
    SMSG_QUERY_TIME_RESPONSE, SMSG_TEXT_EMOTE, SMSG_UPDATE_OBJECT,
};

pub async fn handle_received_client_opcodes(
    client: &mut Client,
    clients: &mut [Client],
    creatures: &mut [Creature],
    db: &mut WorldDatabase,
    move_to_character_screen: &mut bool,
) {
    let guid = client.character().guid;

    while let Ok(opcode) = client.received_messages().try_recv() {
        if let Some(info) = opcode.movement_info() {
            client.character_mut().info = info.clone();
        }

        match opcode {
            ClientOpcodeMessage::CMSG_AREATRIGGER(c) => {
                let pos = Position {
                    map: client.character().map,
                    x: client.character().info.position.x,
                    y: client.character().info.position.y,
                    z: client.character().info.position.z,
                    orientation: client.character().info.orientation,
                };
                match wow_world_base::vanilla::trigger::verify_trigger(pos, c.trigger_id) {
                    wow_world_base::vanilla::trigger::TriggerResult::NotFound => {
                        client
                            .send_system_message(format!("Trigger {} not found", c.trigger_id))
                            .await;
                    }
                    wow_world_base::vanilla::trigger::TriggerResult::NotInsideTrigger(_) => {
                        client
                            .send_system_message(format!("Not inside trigger {}", c.trigger_id))
                            .await;
                    }
                    wow_world_base::vanilla::trigger::TriggerResult::Success(t) => {
                        client
                            .send_system_message(format!("Inside trigger {}", c.trigger_id))
                            .await;
                        for trigger in t.1 {
                            match trigger {
                                Trigger::Inn => {
                                    client.send_system_message("Inside inn").await;
                                }
                                Trigger::Quest { quest_id } => {
                                    client
                                        .send_system_message(format!(
                                            "    Inside quest id {}",
                                            quest_id
                                        ))
                                        .await;
                                }
                                Trigger::Teleport { location, .. } => {
                                    client.send_system_message("    Inside teleport").await;
                                    world_handler::prepare_teleport(*location, client).await
                                }
                            }
                        }
                    }
                }
            }
            ClientOpcodeMessage::CMSG_ITEM_QUERY_SINGLE(c) => {
                let item = wow_items::vanilla::lookup_item(c.item);
                match item {
                    None => {
                        client
                            .send_message(SMSG_ITEM_QUERY_SINGLE_RESPONSE {
                                item: c.item | 0x80000000,
                                found: None,
                            })
                            .await;
                    }
                    Some(item) => {
                        println!("Sending response for {}", item.name());
                        client.send_message(item_to_query_response(item)).await;
                    }
                }
            }
            ClientOpcodeMessage::CMSG_ITEM_NAME_QUERY(c) => {
                let item = wow_items::vanilla::lookup_item(c.item);
                match item {
                    None => {}
                    Some(item) => client.send_message(item_to_name_query_response(item)).await,
                }
            }

            ClientOpcodeMessage::CMSG_NAME_QUERY(c) => {
                let character = db.get_character_by_guid(c.guid);

                client
                    .send_message(SMSG_NAME_QUERY_RESPONSE {
                        guid: c.guid,
                        character_name: character.name,
                        realm_name: "".to_string(),
                        race: character.race_class.race().into(),
                        gender: character.gender.into(),
                        class: character.race_class.class(),
                    })
                    .await;
            }
            ClientOpcodeMessage::CMSG_CREATURE_QUERY(c) => {
                if let Some(creature) = creatures.iter().find(|a| a.entry == c.creature) {
                    client
                        .send_message(SMSG_CREATURE_QUERY_RESPONSE {
                            creature_entry: c.creature,
                            found: Some(SMSG_CREATURE_QUERY_RESPONSE_found {
                                name1: creature.name.clone(),
                                name2: "".to_string(),
                                name3: "".to_string(),
                                name4: "".to_string(),
                                sub_name: "".to_string(),
                                type_flags: 0,
                                creature_type: 0,
                                creature_family: CreatureFamily::None,
                                creature_rank: 0,
                                unknown0: 0,
                                spell_data_id: 0,
                                display_id: 0,
                                civilian: 0,
                                racial_leader: 0,
                            }),
                        })
                        .await;
                }
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
                let p = position_from_str(&c.name);
                if let Some(p) = p {
                    world_handler::prepare_teleport(p, client).await;
                } else {
                    client
                        .send_system_message(format!("Location not found: '{}'", c.name))
                        .await;
                }
            }
            ClientOpcodeMessage::MSG_MOVE_WORLDPORT_ACK => {
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
                    gm_command::gm_command(client, clients, c.message.trim_start_matches('.'), db)
                        .await;

                    return;
                }

                handle_message(client, clients, c).await;
            }
            ClientOpcodeMessage::CMSG_LOGOUT_REQUEST => {
                client
                    .send_message(SMSG_LOGOUT_RESPONSE {
                        result: LogoutResult::Success,
                        speed: LogoutSpeed::Instant,
                    })
                    .await;

                *move_to_character_screen = true;
                client.status = CharacterScreenProgress::CharacterScreen;

                db.replace_character_data(client.character().clone());

                client.send_message(SMSG_LOGOUT_COMPLETE {}).await;
            }
            ClientOpcodeMessage::CMSG_SET_SELECTION(c) => {
                client.character_mut().target = c.target;
            }
            ClientOpcodeMessage::CMSG_QUERY_TIME => {
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
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_FORWARD_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_BACKWARD(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_BACKWARD_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_STOP_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_STRAFE_LEFT(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_STRAFE_LEFT_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_STRAFE_RIGHT(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_STRAFE_RIGHT_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_STRAFE(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_STOP_STRAFE_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_JUMP(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_JUMP_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_TURN_LEFT(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_TURN_LEFT_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_TURN_RIGHT(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_TURN_RIGHT_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_TURN(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_STOP_TURN_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_PITCH_UP(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_PITCH_UP_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_PITCH_DOWN(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_PITCH_DOWN_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_PITCH(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_STOP_PITCH_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_RUN_MODE(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_SET_RUN_MODE_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_WALK_MODE(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_SET_WALK_MODE_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_FALL_LAND(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_FALL_LAND_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_START_SWIM(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_START_SWIM_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_SWIM(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_STOP_SWIM_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_FACING(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_SET_FACING_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_PITCH(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_SET_PITCH_Server { guid, info: c.info }.into(),
                    clients,
                )
                .await
            }
            ClientOpcodeMessage::MSG_MOVE_HEARTBEAT(c) => {
                client.set_movement_info(c.info.clone());
                send_movement_to_clients(
                    MSG_MOVE_HEARTBEAT_Server { guid, info: c.info }.into(),
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
            ClientOpcodeMessage::CMSG_ATTACKSWING(c) => {
                client.character_mut().target = c.guid;
                client.character_mut().attacking = true;
                if client.character().auto_attack_timer > UNARMED_SPEED {
                    continue;
                }
                client.character_mut().auto_attack_timer = UNARMED_SPEED;

                send_to_all(
                    SMSG_ATTACKSTART {
                        attacker: guid,
                        victim: client.character().target,
                    },
                    client,
                    clients,
                )
                .await;

                send_to_all(
                    SMSG_ATTACKERSTATEUPDATE {
                        hit_info: HitInfo::CriticalHit,
                        attacker: guid,
                        target: client.character().target,
                        total_damage: 1337,
                        damages: vec![DamageInfo {
                            spell_school_mask: 0,
                            damage_float: 1332.0,
                            damage_uint: 1332,
                            absorb: 0,
                            resist: 0,
                        }],
                        unknown1: 0,
                        spell_id: 0,
                        damage_state: 0,
                        blocked_amount: 0,
                    },
                    client,
                    clients,
                )
                .await;
            }
            ClientOpcodeMessage::CMSG_ATTACKSTOP => {
                client.character_mut().attacking = false;

                send_to_all(
                    SMSG_ATTACKSTOP {
                        player: guid,
                        enemy: client.character().target,
                        unknown1: 0,
                    },
                    client,
                    clients,
                )
                .await;
            }
            ClientOpcodeMessage::CMSG_SWAP_INV_ITEM(c) => {
                client
                    .character_mut()
                    .inventory
                    .swap(c.source_slot, c.destination_slot);
                let mut player = UpdatePlayerBuilder::new()
                    .set_player_field_inv(
                        c.source_slot,
                        client
                            .character()
                            .inventory
                            .get(c.source_slot)
                            .map(|a| a.guid)
                            .unwrap_or(Guid::zero()),
                    )
                    .set_player_field_inv(
                        c.destination_slot,
                        client
                            .character()
                            .inventory
                            .get(c.destination_slot)
                            .map(|a| a.guid)
                            .unwrap_or(Guid::zero()),
                    );

                for (i, (item, _)) in client.character().inventory.equipment().iter().enumerate() {
                    let (item, random_property, creator) = if let Some(item) = item {
                        (
                            item.item.entry(),
                            item.item.random_property() as u32,
                            item.creator,
                        )
                    } else {
                        (0, 0, Guid::zero())
                    };
                    if let Ok(index) = VisibleItemIndex::try_from(i) {
                        let visible_item =
                            VisibleItem::new(creator, item, [0, 0], random_property, 0);
                        player = player.set_player_visible_item(visible_item, index);
                    }
                }

                send_to_all(
                    SMSG_UPDATE_OBJECT {
                        has_transport: 0,
                        objects: vec![Object {
                            update_type: Object_UpdateType::Values {
                                guid1: guid,
                                mask1: UpdateMask::Player(player.finalize()),
                            },
                        }],
                    },
                    client,
                    clients,
                )
                .await;
            }
            ClientOpcodeMessage::CMSG_TEXT_EMOTE(v) => {
                client
                    .send_system_message(format!("{}, {:#08X}", v.text_emote, v.emote))
                    .await;

                send_to_all(
                    SMSG_EMOTE {
                        emote: v.text_emote.to_emote(),
                        guid,
                    },
                    client,
                    clients,
                )
                .await;

                send_to_all(
                    SMSG_TEXT_EMOTE {
                        guid,
                        text_emote: v.text_emote,
                        emote: v.emote,
                        name: "".to_string(),
                    },
                    client,
                    clients,
                )
                .await;
            }
            v => {
                write_test(&v);
            }
        }
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

pub(crate) fn write_test(msg: &ClientOpcodeMessage) {
    if let Some(contents) = msg.to_test_case_string() {
        let name = msg.message_name();
        if let Some(path) = find_wowm_file(name) {
            println!("Added {name} to {path}", path = path.display());
            append_string_to_file("\n", &path);
            append_string_to_file(&contents, &path);
        } else {
            let path = Path::new("./tests.wowm");
            println!("Added {name} to {path}", path = path.display());
            append_string_to_file("\n", path);
            append_string_to_file(&contents, path);
        }
    } else {
        dbg!(&msg);
    }
}

fn find_wowm_file(name: &str) -> Option<PathBuf> {
    for file in WalkDir::new(Path::new("../wow_messages/wow_message_parser/wowm"))
        .into_iter()
        .filter_map(|a| a.ok())
    {
        let Ok(contents) = read_to_string(file.path()) else {
            continue;
        };

        if contents.contains(name) {
            return Some(file.path().to_path_buf());
        }
    }

    None
}
