use crate::world::character::Character;
use crate::world::character_screen_handler::handle_character_screen_opcodes;
use crate::world::client::{CharacterScreenProgress, Client};
use crate::world::creature::Creature;
use crate::world::database::WorldDatabase;
use crate::world::world_opcode_handler;
use std::convert::TryInto;
use tokio::sync::mpsc::Receiver;
use wow_vanilla_common::class::{get_display_id_for_player, get_power_for_class};
use wow_vanilla_common::factions::get_race_faction;
use wow_vanilla_common::position::{get_position_from_str, Position};
use wow_vanilla_common::race::get_race_scale;
use wow_vanilla_common::range::trace_point_2d;
use wow_vanilla_common::{
    Map, DEFAULT_RUNNING_BACKWARDS_SPEED, DEFAULT_TURN_SPEED, DEFAULT_WALKING_SPEED,
};
use wow_world_messages::vanilla::opcodes::ServerOpcodeMessage;
use wow_world_messages::vanilla::{
    Language, MSG_MOVE_TELEPORT_ACK_Server, MovementBlock, MovementBlock_MovementFlags,
    MovementBlock_UpdateFlag, MovementBlock_UpdateFlag_Living, MovementInfo,
    MovementInfo_MovementFlags, Object, ObjectType, Object_UpdateType, PlayerChatTag,
    SMSG_MESSAGECHAT_ChatType, Vector3d, SMSG_ACCOUNT_DATA_TIMES, SMSG_DESTROY_OBJECT,
    SMSG_FORCE_RUN_SPEED_CHANGE, SMSG_LOGIN_SETTIMESPEED, SMSG_LOGIN_VERIFY_WORLD,
    SMSG_MESSAGECHAT, SMSG_NEW_WORLD, SMSG_SPLINE_SET_RUN_SPEED, SMSG_TRANSFER_PENDING,
    SMSG_TUTORIAL_FLAGS, SMSG_UPDATE_OBJECT,
};
use wow_world_messages::vanilla::{UpdateMask, UpdatePlayer};
use wow_world_messages::{DateTime, Guid};

#[derive(Debug)]
pub struct World {
    clients: Vec<Client>,
    clients_on_character_screen: Vec<Client>,
    clients_waiting_to_join: Receiver<Client>,

    creatures: Vec<Creature>,

    locations: Vec<(Position, String)>,
}

impl World {
    pub fn new(rx: Receiver<Client>) -> Self {
        let locations = read_locations();

        Self {
            clients: vec![],
            clients_on_character_screen: vec![],
            clients_waiting_to_join: rx,
            creatures: vec![Creature::new("Thing")],
            locations,
        }
    }

    pub async fn tick(&mut self, db: WorldDatabase) {
        while let Ok(c) = self.clients_waiting_to_join.try_recv() {
            self.clients_on_character_screen.push(c);
        }

        for client in &mut self.clients_on_character_screen {
            handle_character_screen_opcodes(client, db.clone()).await;
        }

        while let Some(i) = self
            .clients_on_character_screen
            .iter()
            .position(|a| a.status == CharacterScreenProgress::WaitingToLogIn)
        {
            let mut c = self.clients_on_character_screen.remove(i);

            for client in &mut self.clients {
                announce_character_login(client, c.character()).await;
            }

            for client in &mut self.clients {
                announce_character_login(&mut c, client.character()).await;
            }

            for creature in &self.creatures {
                c.send_message(creature.to_message()).await;
            }

            self.clients.push(c);
        }

        let mut move_to_character_screen = Vec::with_capacity(128);

        for i in 0..self.clients.len() {
            let mut client = self.clients.remove(i);
            world_opcode_handler::handle_received_client_opcodes(
                &mut client,
                &mut self.clients,
                &mut self.creatures,
                db.clone(),
                &self.locations,
                &mut move_to_character_screen,
            )
            .await;

            self.clients.insert(i, client);
        }

        for guid in move_to_character_screen {
            let i = self
                .clients
                .iter()
                .position(|a| a.character().guid == guid)
                .unwrap();
            let c = self.clients.remove(i);

            self.clients_on_character_screen.push(c);

            for a in &mut self.clients {
                a.send_message(SMSG_DESTROY_OBJECT { guid }).await;
            }
        }

        while let Some((i, _)) = self
            .clients_on_character_screen
            .iter()
            .enumerate()
            .find(|(_, a)| a.reader_handle.is_finished())
        {
            self.clients_on_character_screen.remove(i);
        }
    }
}

pub fn get_self_update_object_create_object2(character: &Character) -> SMSG_UPDATE_OBJECT {
    let mut m = get_update_object_create_object2(character);

    match &mut m.objects[0].update_type {
        Object_UpdateType::CreateObject2 { movement2, .. } => {
            movement2.update_flag = movement2.update_flag.clone().set_SELF();
        }
        _ => unreachable!(),
    }

    m
}

pub fn get_update_object_create_object2(character: &Character) -> SMSG_UPDATE_OBJECT {
    SMSG_UPDATE_OBJECT {
        has_transport: 0,
        objects: vec![Object {
            update_type: Object_UpdateType::CreateObject2 {
                guid3: character.guid,
                mask2: get_update_object_player(character),
                movement2: MovementBlock {
                    update_flag: MovementBlock_UpdateFlag::new_LIVING(
                        MovementBlock_UpdateFlag_Living::Living {
                            backwards_running_speed: DEFAULT_RUNNING_BACKWARDS_SPEED,
                            backwards_swimming_speed: 0.0,
                            fall_time: 0.0,
                            flags: MovementBlock_MovementFlags::empty(),
                            living_orientation: character.info.orientation,
                            living_position: character.info.position,
                            running_speed: character.movement_speed,
                            swimming_speed: 0.0,
                            timestamp: 0,
                            turn_rate: DEFAULT_TURN_SPEED,
                            walking_speed: DEFAULT_WALKING_SPEED,
                        },
                    ),
                },
                object_type: ObjectType::Player,
            },
        }],
    }
}

fn get_update_object_player(character: &Character) -> UpdateMask {
    UpdateMask::Player(
        UpdatePlayer::new()
            .set_object_GUID(character.guid)
            .set_object_SCALE_X(get_race_scale(character.race, character.gender))
            .set_unit_BYTES_0(
                character.race,
                character.class,
                character.gender,
                get_power_for_class(character.class),
            )
            .set_player_BYTES_2(character.facialhair, 0, 0, 2)
            .set_player_FEATURES(
                character.skin,
                character.face,
                character.hairstyle,
                character.haircolor,
            )
            .set_unit_BASE_HEALTH(character.base_health())
            .set_unit_HEALTH(character.max_health())
            .set_unit_MAXHEALTH(character.max_health())
            .set_unit_LEVEL(character.level as i32)
            .set_unit_AGILITY(character.agility())
            .set_unit_STRENGTH(character.strength())
            .set_unit_STAMINA(character.stamina())
            .set_unit_INTELLECT(character.intellect())
            .set_unit_SPIRIT(character.spirit())
            .set_unit_FACTIONTEMPLATE(get_race_faction(character.race))
            .set_unit_DISPLAYID(get_display_id_for_player(character.race, character.gender))
            .set_unit_NATIVEDISPLAYID(get_display_id_for_player(character.race, character.gender))
            .set_unit_TARGET(character.target),
    )
}

pub async fn announce_character_login(client: &mut Client, character: &Character) {
    let m = get_update_object_create_object2(character);

    client.send_message(m).await;
}

pub fn get_client_login_messages(character: &Character) -> [ServerOpcodeMessage; 6] {
    let mut v = Vec::with_capacity(6);

    let year = 22;
    let month = 7;
    let month_day = 12;
    let week_day = 3;
    let hour = 8;
    let minute = 10;
    v.push(ServerOpcodeMessage::SMSG_LOGIN_SETTIMESPEED(
        SMSG_LOGIN_SETTIMESPEED {
            datetime: DateTime::new(
                year,
                month.try_into().unwrap(),
                month_day,
                week_day.try_into().unwrap(),
                hour,
                minute,
            ),
            timescale: 1.0 / 60.0,
        },
    ));

    v.push(ServerOpcodeMessage::SMSG_LOGIN_VERIFY_WORLD(
        SMSG_LOGIN_VERIFY_WORLD {
            map: character.map,
            position: character.info.position,
            orientation: character.info.orientation,
        },
    ));

    v.push(ServerOpcodeMessage::SMSG_ACCOUNT_DATA_TIMES(
        SMSG_ACCOUNT_DATA_TIMES { data: [0; 32] },
    ));

    v.push(ServerOpcodeMessage::SMSG_TUTORIAL_FLAGS(
        SMSG_TUTORIAL_FLAGS {
            tutorial_data: [
                0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF,
            ],
        },
    ));

    v.push(get_self_update_object_create_object2(character).into());

    v.push(ServerOpcodeMessage::SMSG_MESSAGECHAT(SMSG_MESSAGECHAT {
        chat_type: SMSG_MESSAGECHAT_ChatType::System {
            sender2: Guid::new(0),
        },
        language: Language::Universal,
        message: "Patch 1.12: Drums of War is now live!".to_string(),
        tag: PlayerChatTag::None,
    }));

    v.try_into().unwrap()
}

pub async fn gm_command(
    client: &mut Client,
    clients: &mut [Client],
    message: &str,
    locations: &[(Position, String)],
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

        prepare_teleport(p, client).await;
    } else if message == "next" {
        let p = locations[client.location_index].clone();
        client.location_index += 1;

        client
            .send_system_message(format!("Teleporting to '{}'", p.1))
            .await;

        prepare_teleport(p.0, client).await;
        return;
    } else if message == "prev" {
        let p = locations[client.location_index].clone();
        if client.location_index != 0 {
            client.location_index -= 1;
        }

        client
            .send_system_message(format!("Teleporting to '{}'", p.1))
            .await;

        prepare_teleport(p.0, client).await;
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
    } else if let Some((_, location)) = message.split_once("tp") {
        let location = location.trim();
        let p = get_position_from_str(location);

        if let Some(p) = p {
            prepare_teleport(p, client).await;
        } else {
            client
                .send_system_message(format!("Location not found: '{}'", location))
                .await;
        }

        return;
    } else if let Some((_, locations)) = message.split_once("go") {
        let locations = locations.trim();
        let coords: Vec<&str> = locations.split_whitespace().collect();

        match coords.len() {
            0 => {
                if client.character().target != 0_u64.into() {
                    if let Some(c) = clients
                        .iter()
                        .find(|a| a.character().guid == client.character().target)
                    {
                        prepare_teleport(c.position(), client).await;
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
                    prepare_teleport(c.position(), client).await;
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

                prepare_teleport(p, client).await;
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
    } else if let Some((_, speed)) = message.split_once("speed") {
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
    } else if let Some((_, location)) = message.split_once("mark") {
        let name = location.trim();

        let mut msg = String::with_capacity(128);

        use crate::file_utils::append_string_to_file;
        use std::fmt::Write;
        use std::path::Path;
        writeln!(
            msg,
            "GenPos::new(Position::new(Map::{:?}, {}, {}, {}, {}), vec![\"{}\"]),",
            client.character().map,
            client.character().info.position.x,
            client.character().info.position.y,
            client.character().info.position.z,
            client.character().info.orientation,
            name,
        )
        .unwrap();

        println!("{} added {}", client.character().name, msg);
        append_string_to_file(&msg, Path::new("unadded_locations.txt"));

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
    } else if let Some((_, distance)) = message.split_once("extend") {
        let distance = distance.trim();

        let distance = if let Ok(distance) = distance.parse::<f32>() {
            distance
        } else {
            5.0
        };

        let mut p = client.position();
        let new_location = trace_point_2d(p.x, p.y, p.orientation, distance);

        p.x = new_location.0;
        p.y = new_location.1;

        prepare_teleport(p, client).await;
    }
}

pub async fn prepare_teleport(p: Position, client: &mut Client) {
    if p.map == client.character().map {
        client
            .send_message(MSG_MOVE_TELEPORT_ACK_Server {
                guid: client.character().guid,
                movement_counter: 0,
                info: MovementInfo {
                    flags: MovementInfo_MovementFlags::empty(),
                    timestamp: 0,
                    position: Vector3d {
                        x: p.x,
                        y: p.y,
                        z: p.z,
                    },
                    orientation: p.orientation,
                    fall_time: 0.0,
                },
            })
            .await;
    } else {
        client
            .send_message(SMSG_TRANSFER_PENDING {
                map: p.map,
                has_transport: None,
            })
            .await;

        client
            .send_message(SMSG_NEW_WORLD {
                map: p.map,
                position: Vector3d {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                },
                orientation: p.orientation,
            })
            .await;

        client.character_mut().info.position.x = p.x;
        client.character_mut().info.position.y = p.y;
        client.character_mut().info.position.z = p.z;
        client.character_mut().info.orientation = p.orientation;
        client.character_mut().map = p.map;
        client.in_process_of_teleport = true;
    }
}

fn read_locations() -> Vec<(Position, String)> {
    let b = include_str!("../../../wow_vanilla_common/locations.txt");
    let mut v = Vec::new();

    for line in b.lines() {
        if line.is_empty() {
            continue;
        }

        let coords: Vec<&str> = line.split(',').collect();
        let map = coords[0].trim().parse::<u32>().unwrap();
        let map = Map::try_from(map).unwrap();
        let x = coords[1].trim().parse::<f32>().unwrap();
        let y = coords[2].trim().parse::<f32>().unwrap();
        let z = coords[3].trim().parse::<f32>().unwrap();
        let description = coords[4].to_string().replace('\"', "");

        v.push((Position::new(map, x, y, z, 0.0), description));
    }

    v
}
