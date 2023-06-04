pub(crate) mod gm_command;

use crate::world::character::Character;
use crate::world::character_screen_handler::handle_character_screen_opcodes;
use crate::world::client::{CharacterScreenClient, CharacterScreenProgress, Client};
use crate::world::creature::Creature;
use crate::world::database::WorldDatabase;
use crate::world::world_opcode_handler;
use std::collections::BTreeSet;
use std::convert::TryInto;
use tokio::sync::mpsc::Receiver;
use wow_world_base::combat::UNARMED_SPEED;
use wow_world_base::movement::{
    DEFAULT_RUNNING_BACKWARDS_SPEED, DEFAULT_TURN_SPEED, DEFAULT_WALKING_SPEED,
};
use wow_world_base::vanilla::position::Position;
use wow_world_base::vanilla::{HitInfo, Map};
use wow_world_messages::vanilla::opcodes::ServerOpcodeMessage;
use wow_world_messages::vanilla::UpdateMask;
use wow_world_messages::vanilla::{
    DamageInfo, InitialSpell, Language, MSG_MOVE_TELEPORT_ACK_Server, MovementBlock,
    MovementBlock_MovementFlags, MovementBlock_UpdateFlag, MovementBlock_UpdateFlag_Living,
    MovementInfo, MovementInfo_MovementFlags, Object, ObjectType, Object_UpdateType, PlayerChatTag,
    SMSG_MESSAGECHAT_ChatType, SkillInfo, SkillInfoIndex, UpdateItemBuilder, UpdatePlayerBuilder,
    Vector3d, VisibleItem, VisibleItemIndex, SMSG_ACCOUNT_DATA_TIMES, SMSG_ATTACKERSTATEUPDATE,
    SMSG_DESTROY_OBJECT, SMSG_INITIAL_SPELLS, SMSG_LOGIN_SETTIMESPEED, SMSG_LOGIN_VERIFY_WORLD,
    SMSG_MESSAGECHAT, SMSG_NEW_WORLD, SMSG_TRANSFER_PENDING, SMSG_TUTORIAL_FLAGS,
    SMSG_UPDATE_OBJECT,
};
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
            .position(|a| matches!(a.status, CharacterScreenProgress::WaitingToLogIn(_)))
        {
            let c = self.clients_on_character_screen.remove(i);
            let character = match c.status {
                CharacterScreenProgress::WaitingToLogIn(c) => db.get_character_by_guid(c),
                _ => unreachable!(),
            };
            let mut c = c.into_client(character);

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
                client.character_mut().auto_attack_timer = UNARMED_SPEED;
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
            movement2.update_flag = movement2.update_flag.clone().set_self();
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
                    update_flag: MovementBlock_UpdateFlag::new_living(
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
        .set_object_guid(character.guid)
        .set_object_scale_x(
            character
                .race_class
                .to_race_class()
                .0
                .race_scale(character.gender),
        )
        .set_unit_bytes_0(
            character.race_class.race().into(),
            character.race_class.class(),
            character.gender.into(),
            character.race_class.class().power_type(),
        )
        .set_player_bytes_2(character.facialhair, 0, 0, 2)
        .set_player_features(
            character.skin,
            character.face,
            character.hairstyle,
            character.haircolor,
        )
        .set_unit_base_health(character.base_health())
        .set_unit_health(character.max_health())
        .set_unit_maxhealth(character.max_health())
        .set_unit_level(character.level.as_int() as i32)
        .set_unit_agility(character.agility())
        .set_unit_strength(character.strength())
        .set_unit_stamina(character.stamina())
        .set_unit_intellect(character.intellect())
        .set_unit_spirit(character.spirit())
        .set_unit_factiontemplate(character.race_class.race().faction_id().as_int() as i32)
        .set_unit_displayid(character.race_class.race().display_id(character.gender))
        .set_unit_nativedisplayid(character.race_class.race().display_id(character.gender))
        .set_unit_target(character.target);

    for (i, (item, slot)) in character.inventory.all_slots().iter().enumerate() {
        if let Some(item) = item {
            if let Ok(index) = VisibleItemIndex::try_from(i) {
                let visible_item = VisibleItem::new(
                    Guid::zero(),
                    item.item.entry(),
                    [0, 0],
                    item.item.random_property() as u32,
                    0,
                );
                mask = mask.set_player_visible_item(visible_item, index);
            }
            mask = mask.set_player_field_inv(*slot, item.guid);
        }
    }

    for (i, skill) in character.race_class.starter_skills().iter().enumerate() {
        mask = mask.set_player_skill_info(
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

    let objects = character
        .inventory
        .all_slots()
        .iter()
        .filter_map(|(item, _)| {
            item.map(|item| Object {
                update_type: Object_UpdateType::CreateObject {
                    guid3: item.guid,
                    mask2: UpdateMask::Item(
                        UpdateItemBuilder::new()
                            .set_object_guid(item.guid)
                            .set_object_entry(item.item.entry() as i32)
                            .set_object_scale_x(1.0)
                            .set_item_owner(character.guid)
                            .set_item_contained(character.guid)
                            .set_item_stack_count(item.amount as i32)
                            .set_item_durability(item.item.max_durability())
                            .set_item_maxdurability(item.item.max_durability())
                            .set_item_creator(item.creator)
                            .set_item_stack_count(item.amount as i32)
                            .finalize(),
                    ),
                    movement2: MovementBlock {
                        update_flag: MovementBlock_UpdateFlag::empty(),
                    },
                    object_type: ObjectType::Item,
                },
            })
        })
        .collect();

    v.push(
        SMSG_UPDATE_OBJECT {
            has_transport: 0,
            objects,
        }
        .into(),
    );

    v.push(get_self_update_object_create_object2(character).into());

    v
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
