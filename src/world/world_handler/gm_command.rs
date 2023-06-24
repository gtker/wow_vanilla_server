use crate::world::client::Client;
use crate::world::database::WorldDatabase;
use crate::world::item::Item;
use crate::world::world_handler;
use wow_items::vanilla::lookup_item;
use wow_world_base::geometry::trace_point_2d;
use wow_world_base::vanilla::position::{position_from_str, Position};
use wow_world_base::vanilla::{
    Map, NewItemChatAlert, NewItemCreationType, NewItemSource, SplineFlag, Vector2d, Vector3d,
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
    locations: &[(Position, String)],
    mut db: &mut WorldDatabase,
) {
    if message == "north" || message == "south" || message == "east" || message == "west" {
        let mut p = client.character().info.position;
        match message {
            "north" => p.x += 5.0,
            "south" => p.x -= 5.0,
            "east" => p.y -= 5.0,
            "west" => p.y += 5.0,
            _ => unreachable!(),
        }

        let p = Position::new(
            client.character().map,
            p.x,
            p.y,
            p.z,
            client.character().info.orientation,
        );

        world_handler::prepare_teleport(p, client).await;
    } else if message == "next" {
        let p = locations[client.location_index].clone();
        client.location_index += 1;

        client
            .send_system_message(format!("Teleporting to '{}'", p.1))
            .await;

        world_handler::prepare_teleport(p.0, client).await;
        return;
    } else if message == "prev" {
        let p = locations[client.location_index].clone();
        if client.location_index != 0 {
            client.location_index -= 1;
        }

        client
            .send_system_message(format!("Teleporting to '{}'", p.1))
            .await;

        world_handler::prepare_teleport(p.0, client).await;
        return;
    } else if message == "whereami" {
        client
            .send_system_message(format!(
                "You are on the map '{map}' ({map_int}), x: {x}, y: {y}, z: {z}",
                map = client.character().map,
                map_int = client.character().map.as_int(),
                x = client.character().info.position.x,
                y = client.character().info.position.y,
                z = client.character().info.position.z,
            ))
            .await;

        return;
    } else if let Some(location) = message.strip_prefix("tp") {
        let location = location.trim();
        let p = position_from_str(location);

        if let Some(p) = p {
            world_handler::prepare_teleport(p, client).await;
        } else {
            client
                .send_system_message(format!("Location not found: '{}'", location))
                .await;
        }

        return;
    } else if let Some(locations) = message.strip_prefix("go") {
        let locations = locations.trim();
        let coords: Vec<&str> = locations.split_whitespace().collect();

        match coords.len() {
            0 => {
                if client.character().target != 0_u64.into() {
                    if let Some(c) = clients
                        .iter()
                        .find(|a| a.character().guid == client.character().target)
                    {
                        world_handler::prepare_teleport(c.position(), client).await;
                    } else {
                        client
                            .send_system_message(format!(
                                "Unable to find character with GUID: '{}'",
                                client.character().target
                            ))
                            .await;
                    }
                } else {
                    client
                        .send_system_message(
                            "No target for .go command without arguments.".to_string(),
                        )
                        .await;
                }
            }
            2 => {
                // We can't use only x and y
                client
                    .send_system_message(
                        "Can not teleport with only x and y coordinates".to_string(),
                    )
                    .await;
            }
            1 => {
                let name = locations.trim().to_lowercase();
                if let Some(c) = clients
                    .iter()
                    .find(|a| a.character().name.to_lowercase() == name)
                {
                    world_handler::prepare_teleport(c.position(), client).await;
                } else {
                    client
                        .send_system_message(format!("Unable to find player '{}'", name))
                        .await;
                }
            }
            3 | 4 => {
                let x = coords[0].parse::<f32>();
                let x = match x {
                    Ok(p) => p,
                    Err(_) => {
                        client
                            .send_system_message("invalid x coordinate".to_string())
                            .await;
                        return;
                    }
                };
                let y = coords[1].parse::<f32>();
                let y = match y {
                    Ok(p) => p,
                    Err(_) => {
                        client
                            .send_system_message("invalid y coordinate".to_string())
                            .await;
                        return;
                    }
                };
                let z = coords[2].parse::<f32>();
                let z = match z {
                    Ok(p) => p,
                    Err(_) => {
                        client
                            .send_system_message("invalid z coordinate".to_string())
                            .await;
                        return;
                    }
                };

                let map = if coords.len() == 3 {
                    client.character().map
                } else {
                    let map = coords[3].parse::<u32>();
                    let map = match map {
                        Ok(p) => p,
                        Err(_) => {
                            client.send_system_message("invalid map".to_string()).await;
                            return;
                        }
                    };
                    match Map::try_from(map) {
                        Ok(m) => m,
                        Err(_) => {
                            client.send_system_message("invalid map".to_string()).await;
                            return;
                        }
                    }
                };

                let p = Position::new(map, x, y, z, client.character().info.orientation);

                world_handler::prepare_teleport(p, client).await;
            }
            _ => {
                // Too many args
                client
                    .send_system_message(
                        "Incorrect '.go' command: Too many coordinates".to_string(),
                    )
                    .await;
            }
        }
    } else if let Some(speed) = message.strip_prefix("speed") {
        if let Ok(speed) = speed.trim().parse::<f32>() {
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
        } else {
            client
                .send_system_message(format!("Value '{}' is not a valid number", speed))
                .await;
        }
    } else if let Some(location) = message.strip_prefix("mark") {
        let name = location.trim();

        if name.is_empty() {
            client
                .send_system_message(
                    ".mark a list of names separated by a comma. Like '.mark Honor Hold, HH`",
                )
                .await;
            return;
        }

        let mut msg = String::with_capacity(128);

        use crate::file_utils::append_string_to_file;
        use std::fmt::Write;
        use std::path::Path;

        let names = name.split(",").map(|a| a.trim());

        write!(
            msg,
            "RawPosition::new({}, {}, {}, {}, {}, vec![",
            client.character().map.as_int(),
            client.character().info.position.x,
            client.character().info.position.y,
            client.character().info.position.z,
            client.character().info.orientation,
        )
        .unwrap();

        for name in names {
            write!(msg, "\"{name}\",").unwrap();
        }

        writeln!(
            msg,
            "], ValidVersions::new(false, {tbc}, {vanilla})),",
            tbc = client.character().map.as_int() == 530,
            vanilla =
                client.character().map.as_int() == 571 || client.character().map.as_int() == 530,
        )
        .unwrap();

        println!("{} added {}", client.character().name, msg);
        append_string_to_file(&msg, Path::new("unadded_locations.txt"));

        let msg = format!("You added {}", msg);

        client.send_system_message(msg).await
    } else if message == "range" {
        if let Some(c) = clients
            .iter()
            .find(|a| a.character().guid == client.character().target)
        {
            if let Some(distance) = client.distance_to_center(c) {
                client
                    .send_system_message(format!("Range to target: '{}'", distance))
                    .await;
            } else {
                client
                    .send_system_message("Not on same map, no valid range.".to_string())
                    .await;
            }
        } else if client.character().guid == client.character().target {
            client
                .send_system_message("Distance to self is always 0".to_string())
                .await;
        } else {
            client
                .send_system_message("Unable to find range, no target selected".to_string())
                .await;
        }
    } else if let Some(distance) = message.strip_prefix("extend") {
        let distance = distance.trim();

        let distance = if let Ok(distance) = distance.parse::<f32>() {
            distance
        } else {
            5.0
        };

        let mut p = client.position();
        let new_location = trace_point_2d(Vector2d { x: p.x, y: p.y }, p.orientation, distance);

        p.x = new_location.0;
        p.y = new_location.1;

        world_handler::prepare_teleport(p, client).await;
    } else if let Some(distance) = message.strip_prefix("float") {
        let distance = distance.trim();

        let distance = if let Ok(distance) = distance.parse::<f32>() {
            distance
        } else {
            5.0
        };

        let mut p = client.position();
        p.z = p.z + distance;

        world_handler::prepare_teleport(p, client).await;
    } else if let Some(entry) = message.strip_prefix("additem") {
        let entry = match entry.trim().parse::<i32>() {
            Ok(e) => e,
            Err(_) => 12640, // Lionheart Helm
        };

        let Some(item_entry) = lookup_item(entry as u32) else {
            client.send_system_message(format!("Item entry {entry} not found.")).await;
            return;
        };

        const AMOUNT: u8 = 1;

        let item = Item::new(item_entry, client.character().guid, AMOUNT, &mut db);

        let item_slot = client
            .character_mut()
            .inventory
            .insert_into_first_slot(item);
        let Some(item_slot) = item_slot else {
            client.send_system_message("Unable to add item. No free slots available.").await;
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
                    item: entry as u32,
                    item_suffix_factor: 0,
                    item_random_property_id: 0,
                    item_count: AMOUNT as u32,
                }
                .into(),
            )
            .await;
    } else if message == "move" {
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
