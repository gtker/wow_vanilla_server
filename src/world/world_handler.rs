use crate::world::character::Character;
use crate::world::character_screen_handler::handle_character_screen_opcodes;
use crate::world::client::{CharacterScreenClient, CharacterScreenProgress, Client};
use crate::world::creature::Creature;
use crate::world::database::WorldDatabase;
use crate::world::world_opcode_handler;
use std::collections::BTreeSet;
use std::convert::TryInto;
use tokio::sync::mpsc::Receiver;
use wow_world_base::combat::UNARMED_SPEED_FLOAT;
use wow_world_base::geometry::trace_point_2d;
use wow_world_base::vanilla::position::{position_from_str, Position};
use wow_world_base::vanilla::{
    HitInfo, Map, NewItemChatAlert, NewItemCreationType, NewItemSource, SplineFlag,
};
use wow_world_base::{DEFAULT_RUNNING_BACKWARDS_SPEED, DEFAULT_TURN_SPEED, DEFAULT_WALKING_SPEED};
use wow_world_messages::vanilla::opcodes::ServerOpcodeMessage;
use wow_world_messages::vanilla::{
    CompressedMove, CompressedMove_CompressedMoveOpcode, DamageInfo, InitialSpell, Language,
    MSG_MOVE_TELEPORT_ACK_Server, MonsterMove, MonsterMoveSplines, MonsterMove_MonsterMoveType,
    MovementBlock, MovementBlock_MovementFlags, MovementBlock_UpdateFlag,
    MovementBlock_UpdateFlag_All, MovementBlock_UpdateFlag_Living, MovementInfo,
    MovementInfo_MovementFlags, Object, ObjectType, Object_UpdateType, PlayerChatTag,
    SMSG_MESSAGECHAT_ChatType, SkillInfo, SkillInfoIndex, UpdateItemBuilder, UpdatePlayerBuilder,
    Vector3d, SMSG_ACCOUNT_DATA_TIMES, SMSG_ATTACKERSTATEUPDATE, SMSG_COMPRESSED_MOVES,
    SMSG_DESTROY_OBJECT, SMSG_FORCE_RUN_SPEED_CHANGE, SMSG_INITIAL_SPELLS, SMSG_ITEM_PUSH_RESULT,
    SMSG_LOGIN_SETTIMESPEED, SMSG_LOGIN_VERIFY_WORLD, SMSG_MESSAGECHAT, SMSG_NEW_WORLD,
    SMSG_SPLINE_SET_RUN_SPEED, SMSG_TRANSFER_PENDING, SMSG_TUTORIAL_FLAGS, SMSG_UPDATE_OBJECT,
};
use wow_world_messages::vanilla::{UpdateMask, Vector2d};
use wow_world_messages::{DateTime, Guid};

#[derive(Debug)]
pub struct World {
    clients: Vec<Client>,
    clients_on_character_screen: Vec<CharacterScreenClient>,
    clients_waiting_to_join: Receiver<CharacterScreenClient>,

    creatures: Vec<Creature>,

    locations: Vec<(Position, String)>,
}

impl World {
    pub fn new(rx: Receiver<CharacterScreenClient>) -> Self {
        let locations = read_locations();

        Self {
            clients: vec![],
            clients_on_character_screen: vec![],
            clients_waiting_to_join: rx,
            creatures: vec![Creature::new("Thing")],
            locations,
        }
    }

    pub async fn tick(&mut self, db: &mut WorldDatabase) {
        while let Ok(c) = self.clients_waiting_to_join.try_recv() {
            self.clients_on_character_screen.push(c);
        }

        for client in &mut self.clients_on_character_screen {
            handle_character_screen_opcodes(client, db).await;
        }

        while let Some(i) = self
            .clients_on_character_screen
            .iter()
            .position(|a| a.status == CharacterScreenProgress::WaitingToLogIn)
        {
            let c = self.clients_on_character_screen.remove(i);
            let mut c = c.into_client();

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

        let mut indices_to_move_to_character_screen = BTreeSet::new();
        let mut move_to_character_screen = false;

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
            client.character_mut().update_auto_attack_timer();

            if client.character().attacking && client.character().auto_attack_timer <= 0.0 {
                client.character_mut().auto_attack_timer = UNARMED_SPEED_FLOAT;
                let msg = SMSG_ATTACKERSTATEUPDATE {
                    hit_info: HitInfo::CriticalHit,
                    attacker: client.character().guid,
                    target: client.character().target,
                    total_damage: 1332,
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
                };

                client.send_message(msg.clone()).await;

                for c in &mut self.clients {
                    c.send_message(msg.clone()).await;
                }
            }

            if move_to_character_screen {
                indices_to_move_to_character_screen.insert(i);
            }

            self.clients.insert(i, client);
        }

        for i in indices_to_move_to_character_screen.iter().rev() {
            let c = self.clients.remove(*i);
            for a in &mut self.clients {
                a.send_message(SMSG_DESTROY_OBJECT {
                    guid: c.character().guid,
                })
                .await;
            }

            let c = c.into_character_screen_client();
            self.clients_on_character_screen.push(c);
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
    let mut mask = UpdatePlayerBuilder::new()
        .set_object_GUID(character.guid)
        .set_object_SCALE_X(
            character
                .race_class
                .to_race_class()
                .0
                .race_scale(character.gender),
        )
        .set_unit_BYTES_0(
            character.race_class.race().into(),
            character.race_class.class(),
            character.gender.into(),
            character.race_class.class().power_type(),
        )
        .set_player_BYTES_2(character.facialhair, 0, 0, 2)
        .set_player_FEATURES(
            character.skin,
            character.face,
            character.hairstyle,
            character.haircolor,
        )
        .set_unit_BASE_HEALTH(character.base_health())
        .set_player_VISIBLE_ITEM_1_0(12640) // Lionheart Helm
        .set_player_VISIBLE_ITEM_5_0(11726)
        .set_unit_HEALTH(character.max_health())
        .set_unit_MAXHEALTH(character.max_health())
        .set_unit_LEVEL(character.level.as_int() as i32)
        .set_unit_AGILITY(character.agility())
        .set_unit_STRENGTH(character.strength())
        .set_unit_STAMINA(character.stamina())
        .set_unit_INTELLECT(character.intellect())
        .set_unit_SPIRIT(character.spirit())
        .set_unit_FACTIONTEMPLATE(character.race_class.race().faction_id().as_int() as i32)
        .set_unit_DISPLAYID(character.race_class.race().display_id(character.gender))
        .set_unit_NATIVEDISPLAYID(character.race_class.race().display_id(character.gender))
        .set_unit_TARGET(character.target);

    for (i, skill) in character.race_class.starter_skills().iter().enumerate() {
        mask = mask.set_player_SKILL_INFO(
            SkillInfo::new(*skill, 0, 295, 300, 0, 2),
            SkillInfoIndex::try_from(i as u32).unwrap(),
        );
    }

    UpdateMask::Player(mask.finalize())
}

pub async fn announce_character_login(client: &mut Client, character: &Character) {
    let m = get_update_object_create_object2(character);

    client.send_message(m).await;
}

pub fn get_client_login_messages(character: &Character) -> Vec<ServerOpcodeMessage> {
    let mut v = Vec::with_capacity(16);

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

    v.push(ServerOpcodeMessage::SMSG_MESSAGECHAT(SMSG_MESSAGECHAT {
        chat_type: SMSG_MESSAGECHAT_ChatType::System {
            sender2: Guid::zero(),
        },
        language: Language::Universal,
        message: "Patch 3.3.5: Whatever is now live!".to_string(),
        tag: PlayerChatTag::None,
    }));

    v.push(
        SMSG_INITIAL_SPELLS {
            unknown1: 0,
            initial_spells: character
                .race_class
                .starter_spells()
                .iter()
                .map(|a| InitialSpell {
                    spell_id: *a as u16,
                    unknown1: 0,
                })
                .collect(),
            cooldowns: vec![],
        }
        .into(),
    );

    v.push(get_self_update_object_create_object2(character).into());

    v
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
    } else if let Some(location) = message.strip_prefix("tp") {
        let location = location.trim();
        let p = position_from_str(location);

        if let Some(p) = p {
            prepare_teleport(p, client).await;
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

        prepare_teleport(p, client).await;
    } else if let Some(distance) = message.strip_prefix("float") {
        let distance = distance.trim();

        let distance = if let Ok(distance) = distance.parse::<f32>() {
            distance
        } else {
            5.0
        };

        let mut p = client.position();
        p.z = p.z + distance;

        prepare_teleport(p, client).await;
    } else if message == "item" {
        client
            .send_opcode(
                &SMSG_UPDATE_OBJECT {
                    has_transport: 0,
                    objects: vec![
                        Object {
                            update_type: Object_UpdateType::CreateObject {
                                guid3: Guid::new(1337_1337),
                                mask2: UpdateMask::Item(
                                    UpdateItemBuilder::new()
                                        .set_object_GUID(1337_1337.into())
                                        .set_object_ENTRY(12640)
                                        .set_object_SCALE_X(1.0)
                                        .set_item_OWNER(client.character().guid)
                                        .set_item_CONTAINED(client.character().guid)
                                        .set_item_STACK_COUNT(1)
                                        .set_item_DURABILITY(100)
                                        .set_item_MAXDURABILITY(100)
                                        .finalize(),
                                ),
                                movement2: MovementBlock {
                                    update_flag: MovementBlock_UpdateFlag::empty()
                                        .set_ALL(MovementBlock_UpdateFlag_All { unknown1: 1 }),
                                },
                                object_type: ObjectType::Item,
                            },
                        },
                        Object {
                            update_type: Object_UpdateType::Values {
                                guid1: client.character().guid,
                                mask1: UpdateMask::Player(
                                    UpdatePlayerBuilder::new()
                                        .set_player_FIELD_INV_SLOT_HEAD(1337_1337.into())
                                        .finalize(),
                                ),
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
                    item_slot: 24,
                    item: 12640,
                    item_suffix_factor: 0,
                    item_random_property_id: 0,
                    item_count: 1,
                }
                .into(),
            )
            .await;
    } else if message == "move" {
        let mut splines = MonsterMoveSplines::new();
        splines.splines.push(Vector3d {
            x: -8937.863,
            y: -117.46813,
            z: 82.39997,
        });

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
                                splines,
                            },
                        },
                    },
                    guid: 100.into(),
                }],
            })
            .await;
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
    }

    client.character_mut().info.position.x = p.x;
    client.character_mut().info.position.y = p.y;
    client.character_mut().info.position.z = p.z;
    client.character_mut().info.orientation = p.orientation;
    client.character_mut().map = p.map;
    client.in_process_of_teleport = true;
}

fn read_locations() -> Vec<(Position, String)> {
    let b = "";
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
