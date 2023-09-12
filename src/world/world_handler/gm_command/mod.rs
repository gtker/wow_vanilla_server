mod parser;

use crate::world::client::Client;
use crate::world::database::WorldDatabase;
use crate::world::item::Item;
use crate::world::world_handler;
use crate::world::world_handler::gm_command::parser::GmCommand;
use wow_world_base::vanilla::{
    NewItemChatAlert, NewItemCreationType, NewItemSource, SplineFlag, Vector3d,
};
use wow_world_messages::vanilla::{
    CompressedMove, CompressedMove_CompressedMoveOpcode, MonsterMove, MonsterMove_MonsterMoveType,
    Object, Object_UpdateType, UpdatePlayerBuilder, SMSG_COMPRESSED_MOVES,
    SMSG_FORCE_RUN_SPEED_CHANGE, SMSG_ITEM_PUSH_RESULT, SMSG_SPLINE_SET_RUN_SPEED,
    SMSG_UPDATE_OBJECT,
};

pub async fn gm_command(
    client: &mut Client,
    clients: &mut [Client],
    message: &str,
    mut db: &mut WorldDatabase,
) {
    let command = match GmCommand::from_player_command(message, client, clients) {
        Ok(e) => e,
        Err(e) => {
            client.send_system_message(e).await;
            return;
        }
    };

    match command {
        GmCommand::WhereAmI => {
            client
                .send_system_message(format!(
                    "You are on '{map}' ({map_int}), x: {x}, y: {y}, z: {z}",
                    map = client.character().map,
                    map_int = client.character().map.as_int(),
                    x = client.character().info.position.x,
                    y = client.character().info.position.y,
                    z = client.character().info.position.z,
                ))
                .await;
        }
        GmCommand::Teleport(p) => {
            world_handler::prepare_teleport(p, client).await;
        }
        GmCommand::SetRunSpeed(speed) => {
            client.character_mut().movement_speed = speed;
            client
                .send_message(SMSG_FORCE_RUN_SPEED_CHANGE {
                    guid: client.character().guid,
                    move_event: 0,
                    speed,
                })
                .await;

            for c in clients {
                c.send_message(SMSG_SPLINE_SET_RUN_SPEED {
                    guid: client.character().guid,
                    speed,
                })
                .await;
            }
        }
        GmCommand::Mark { names, p } => {
            use crate::file_utils::append_string_to_file;
            use std::fmt::Write;
            use std::path::Path;

            let mut msg = String::with_capacity(128);

            write!(
                msg,
                "RawPosition::new({}, {}, {}, {}, {}, vec![",
                p.map.as_int(),
                p.x,
                p.y,
                p.z,
                p.orientation,
            )
            .unwrap();

            for name in names {
                write!(msg, "\"{name}\",").unwrap();
            }

            writeln!(
                msg,
                "], ValidVersions::new(false, {tbc}, {vanilla})),",
                tbc = client.character().map.as_int() == 530,
                vanilla = client.character().map.as_int() == 571
                    || client.character().map.as_int() == 530,
            )
            .unwrap();

            println!("{} added {}", client.character().name, msg);
            append_string_to_file(&msg, Path::new("unadded_locations.txt"));

            let msg = format!("You added {}", msg);

            client.send_system_message(msg).await
        }
        GmCommand::RangeToTarget(range) => {
            client
                .send_system_message(format!("Range to target: '{}'", range))
                .await;
        }
        GmCommand::AddItem(item) => {
            const AMOUNT: u8 = 1;

            let item = Item::new(item, client.character().guid, AMOUNT, &mut db);

            let item_slot = client
                .character_mut()
                .inventory
                .insert_into_first_slot(item);
            let Some(item_slot) = item_slot else {
                client
                    .send_system_message("Unable to add item. No free slots available.")
                    .await;
                return;
            };

            client
                .send_opcode(
                    &SMSG_UPDATE_OBJECT {
                        has_transport: 0,
                        objects: vec![
                            item.to_create_item_object(client.character().guid),
                            Object {
                                update_type: Object_UpdateType::Values {
                                    guid1: client.character().guid,
                                    mask1: UpdatePlayerBuilder::new()
                                        .set_player_field_inv(item_slot, item.guid)
                                        .finalize()
                                        .into(),
                                },
                            },
                        ],
                    }
                    .into(),
                )
                .await;

            client
                .send_opcode(
                    &SMSG_ITEM_PUSH_RESULT {
                        guid: client.character().guid,
                        source: NewItemSource::Looted,
                        creation_type: NewItemCreationType::Created,
                        alert_chat: NewItemChatAlert::Show,
                        bag_slot: 0xff,
                        item_slot: item_slot.as_int() as u32,
                        item: item.item.entry(),
                        item_suffix_factor: 0,
                        item_random_property_id: 0,
                        item_count: AMOUNT as u32,
                    }
                    .into(),
                )
                .await;
        }
        GmCommand::MoveNpc => {
            client
                .send_message(SMSG_COMPRESSED_MOVES {
                    moves: vec![CompressedMove {
                        opcode: CompressedMove_CompressedMoveOpcode::SmsgMonsterMove {
                            monster_move: MonsterMove {
                                spline_point: Vector3d {
                                    x: -8938.857,
                                    y: -131.36594,
                                    z: 83.57745,
                                },
                                spline_id: 0,
                                move_type: MonsterMove_MonsterMoveType::Normal {
                                    duration: 0,
                                    spline_flags: SplineFlag::empty(),
                                    splines: vec![Vector3d {
                                        x: -8937.863,
                                        y: -117.46813,
                                        z: 82.39997,
                                    }],
                                },
                            },
                        },
                        guid: 100_u64.into(),
                    }],
                })
                .await;
        }
    }
}
