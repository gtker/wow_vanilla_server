mod parser;

use crate::world::database::WorldDatabase;
use crate::world::world;
use crate::world::world::client::Client;
use crate::world::world::pathfinding_maps::PathfindingMaps;
use crate::world::world_opcode_handler::creature::Creature;
use crate::world::world_opcode_handler::gm_command::parser::GmCommand;
use crate::world::world_opcode_handler::item::{award_item, Item};
use wow_world_base::vanilla::position::Position;
use wow_world_base::vanilla::{SplineFlag, Vector3d};
use wow_world_messages::vanilla::{
    CompressedMove, CompressedMove_CompressedMoveOpcode, MonsterMove, MonsterMove_MonsterMoveType,
    SMSG_COMPRESSED_MOVES, SMSG_FORCE_RUN_SPEED_CHANGE, SMSG_SPLINE_SET_RUN_SPEED,
};

pub async fn gm_command(
    client: &mut Client,
    clients: &mut [Client],
    creatures: &mut [Creature],
    message: &str,
    mut db: &mut WorldDatabase,
    maps: &mut PathfindingMaps,
) {
    let command = match GmCommand::from_player_command(message, client, clients, creatures) {
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
            world::prepare_teleport(p, client).await;
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

            award_item(item, client, clients).await;
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
                        guid: creatures[0].guid,
                    }],
                })
                .await;
        }
        GmCommand::Information(target) => {
            let info = if let Some(target) = clients.iter().find(|a| a.character().guid == target) {
                let name = target.character().name.as_str();
                let guid = target.character().guid;
                let race = target.character().race_class;
                let gender = target.character().gender;
                let level = target.character().level;

                let map = target.character().map;
                let Position { x, y, z, .. } = target.position();

                format!("Player '{name}' ({guid})\nLevel {level} {gender} {race}\n{map} x: {x}, y: {y}, z: {z}")
            } else if let Some(target) = creatures.iter().find(|a| a.guid == target) {
                let name = target.name.as_str();
                let guid = target.guid;

                let map = target.map;
                let Position { x, y, z, .. } = target.position();

                format!("Creature '{name}' ({guid})\n{map} x: {x}, y: {y}, z: {z} (Client movement not supported)")
            } else {
                client
                    .send_system_message(format!("Unable to find target '{target}'"))
                    .await;
                return;
            };

            client.send_system_message(info).await;
        }
        GmCommand::ShouldNotHaveLineOfSight(target) | GmCommand::ShouldHaveLineOfSight(target) => {
            let pos = client.position();
            let o = if let Some(other) = clients.iter().find(|a| a.character().guid == target) {
                other
            } else {
                client
                    .send_system_message(format!("Unable to find target '{target}'"))
                    .await;
                return;
            };
            let other = o.position();

            let f = if let Some(map) = maps.get(&pos.map) {
                let los = map.line_of_sight(pos.into(), other.into()).unwrap();
                if los {
                    client
                        .send_system_message(format!("Has line of sight to {}", o.character().name))
                } else {
                    client.send_system_message(format!(
                        "Has no line of sight to {}",
                        o.character().name
                    ))
                }
            } else {
                client.send_system_message(format!(
                    "Unable to find map '{map}' in pathfinding maps",
                    map = pos.map
                ))
            };

            f.await;
        }
    }
}
