use crate::world::chat::handle_message;
use crate::world::client::{CharacterScreenProgress, Client};
use crate::world::creature::Creature;
use crate::world::database::WorldDatabase;
use crate::world::world_handler;
use crate::world::world_handler::{
    announce_character_login, update_player_builder_visible_id_to_function,
};
use std::time::SystemTime;
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
    SMSG_CREATURE_QUERY_RESPONSE_found, UpdateMask, UpdatePlayerBuilder, SMSG_ATTACKERSTATEUPDATE,
    SMSG_ATTACKSTART, SMSG_ATTACKSTOP, SMSG_CREATURE_QUERY_RESPONSE,
    SMSG_ITEM_QUERY_SINGLE_RESPONSE, SMSG_LOGOUT_COMPLETE, SMSG_LOGOUT_RESPONSE,
    SMSG_NAME_QUERY_RESPONSE, SMSG_PONG, SMSG_QUERY_TIME_RESPONSE, SMSG_UPDATE_OBJECT,
};

pub async fn handle_received_client_opcodes(
    client: &mut Client,
    clients: &mut [Client],
    creatures: &mut [Creature],
    mut db: WorldDatabase,
    locations: &[(Position, String)],
    move_to_character_screen: &mut bool,
) {
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
                client
                    .send_message(SMSG_CREATURE_QUERY_RESPONSE {
                        creature_entry: c.creature,
                        found: Some(SMSG_CREATURE_QUERY_RESPONSE_found {
                            name1: "Ghoul".to_string(),
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
                client.set_movement_info(c.info.clone());
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
            ClientOpcodeMessage::CMSG_ATTACKSWING(c) => {
                client.character_mut().target = c.guid;
                client.character_mut().attacking = true;
                if client.character().auto_attack_timer > UNARMED_SPEED {
                    continue;
                }
                client.character_mut().auto_attack_timer = UNARMED_SPEED;

                for c in &mut *clients {
                    c.send_message(SMSG_ATTACKSTART {
                        attacker: client.character().guid,
                        victim: client.character().target,
                    })
                    .await;
                }
                client
                    .send_message(SMSG_ATTACKSTART {
                        attacker: client.character().guid,
                        victim: client.character().target,
                    })
                    .await;

                client
                    .send_message(SMSG_ATTACKERSTATEUPDATE {
                        hit_info: HitInfo::CriticalHit,
                        attacker: client.character().guid,
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
                    })
                    .await;
                for c in &mut *clients {
                    c.send_message(SMSG_ATTACKERSTATEUPDATE {
                        hit_info: HitInfo::CriticalHit,
                        attacker: client.character().guid,
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
                    })
                    .await;
                }
            }
            ClientOpcodeMessage::CMSG_ATTACKSTOP => {
                client.character_mut().attacking = false;

                for c in &mut *clients {
                    c.send_message(SMSG_ATTACKSTOP {
                        player: client.character().guid,
                        enemy: client.character().target,
                        unknown1: 0,
                    })
                    .await;
                }
                client
                    .send_message(SMSG_ATTACKSTOP {
                        player: client.character().guid,
                        enemy: client.character().target,
                        unknown1: 0,
                    })
                    .await;
            }
            ClientOpcodeMessage::CMSG_SWAP_INV_ITEM(c) => {
                if let Some(source) = client.character_mut().inventory.take(c.source_slot) {
                    if let Some(destination) =
                        client.character_mut().inventory.take(c.destination_slot)
                    {
                        client
                            .character_mut()
                            .inventory
                            .set(c.destination_slot, source);
                        client
                            .character_mut()
                            .inventory
                            .set(c.source_slot, destination);
                    } else {
                        client
                            .character_mut()
                            .inventory
                            .set(c.destination_slot, source);
                        client.character_mut().inventory.clear(c.source_slot);
                    }

                    let mut player = UpdatePlayerBuilder::new()
                        .set_player_field_inv_slot(
                            c.source_slot,
                            client
                                .character()
                                .inventory
                                .get(c.source_slot)
                                .map(|a| a.guid)
                                .unwrap_or(Guid::zero()),
                        )
                        .set_player_field_inv_slot(
                            c.destination_slot,
                            client
                                .character()
                                .inventory
                                .get(c.destination_slot)
                                .map(|a| a.guid)
                                .unwrap_or(Guid::zero()),
                        );

                    for (i, (item, _)) in
                        client.character().inventory.equipment().iter().enumerate()
                    {
                        let item = if let Some(item) = item {
                            item.item.entry() as i32
                        } else {
                            0
                        };
                        if let Some(visible) = update_player_builder_visible_id_to_function(i) {
                            player = visible(player, item);
                        }
                    }

                    let update = SMSG_UPDATE_OBJECT {
                        has_transport: 0,
                        objects: vec![Object {
                            update_type: Object_UpdateType::Values {
                                guid1: client.character().guid,
                                mask1: UpdateMask::Player(player.finalize()),
                            },
                        }],
                    };

                    client.send_message(update.clone()).await;

                    for c in &mut *clients {
                        c.send_message(update.clone()).await;
                    }
                }
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
