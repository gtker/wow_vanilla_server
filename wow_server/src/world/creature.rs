use wow_vanilla_common::position::{get_position, PositionIdentifier};
use wow_vanilla_common::{DEFAULT_RUNNING_SPEED, DEFAULT_TURN_SPEED, DEFAULT_WALKING_SPEED};
use wow_world_messages::vanilla::{
    MovementBlock, MovementBlock_UpdateFlag, MovementBlock_UpdateFlag_Living, MovementInfo, Object,
    ObjectType, Object_UpdateType, Vector3d, SMSG_UPDATE_OBJECT,
};
use wow_world_messages::vanilla::{UpdateMask, UpdateUnit};
use wow_world_messages::Guid;

#[derive(Debug)]
pub struct Creature {
    name: String,
    guid: Guid,
    info: MovementInfo,
    level: u8,
    display_id: u16,
}

impl Creature {
    pub fn new(name: impl Into<String>) -> Self {
        let p = get_position(PositionIdentifier::HumanStartZone);

        Self {
            name: name.into(),
            guid: Guid::new(100),
            info: MovementInfo {
                flags: Default::default(),
                timestamp: 0,
                position: Vector3d {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                },
                orientation: p.orientation,
                fall_time: 0.0,
            },
            level: 1,
            display_id: 646,
        }
    }

    pub fn to_message(&self) -> SMSG_UPDATE_OBJECT {
        SMSG_UPDATE_OBJECT {
            has_transport: 0,
            objects: vec![Object {
                update_type: Object_UpdateType::CreateObject2 {
                    guid3: self.guid,
                    mask2: UpdateMask::Unit(
                        UpdateUnit::new()
                            .set_unit_HEALTH(100)
                            .set_unit_MAXHEALTH(100)
                            .set_object_GUID(self.guid)
                            .set_unit_DISPLAYID(self.display_id.into())
                            .set_object_SCALE_X(1.0)
                            .set_unit_LEVEL(self.level.into())
                            .set_unit_FACTIONTEMPLATE(16),
                    ),
                    movement2: MovementBlock {
                        update_flag: MovementBlock_UpdateFlag::new_LIVING(
                            MovementBlock_UpdateFlag_Living::Living {
                                backwards_running_speed: 0.0,
                                backwards_swimming_speed: 0.0,
                                fall_time: 0.0,
                                flags: Default::default(),
                                living_orientation: 0.0,
                                living_position: self.info.position,
                                running_speed: DEFAULT_RUNNING_SPEED,
                                swimming_speed: 0.0,
                                timestamp: 0,
                                turn_rate: DEFAULT_TURN_SPEED,
                                walking_speed: DEFAULT_WALKING_SPEED,
                            },
                        ),
                    },
                    object_type: ObjectType::Unit,
                },
            }],
        }
    }
}
