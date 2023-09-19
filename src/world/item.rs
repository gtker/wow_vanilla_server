use crate::world::client::Client;
use crate::world::database::WorldDatabase;
use wow_world_base::vanilla::{
    BagFamily, NewItemChatAlert, NewItemCreationType, NewItemSource, ObjectType,
};
use wow_world_messages::vanilla::{
    MovementBlock, MovementBlock_UpdateFlag, Object, Object_UpdateType, UpdateItemBuilder,
    UpdatePlayerBuilder, SMSG_ITEM_PUSH_RESULT, SMSG_UPDATE_OBJECT,
};
use wow_world_messages::Guid;

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub item: &'static wow_world_base::vanilla::Item,
    pub guid: Guid,
    pub amount: u8,
    pub creator: Guid,
}

impl Item {
    pub fn new(
        item: &'static wow_world_base::vanilla::Item,
        creator: Guid,
        amount: u8,
        db: &mut WorldDatabase,
    ) -> Self {
        Self {
            item,
            guid: db.new_guid().into(),
            amount,
            creator,
        }
    }

    pub fn to_create_item_object(&self, item_owner: Guid) -> Object {
        let object_type = match self.item.bag_family() {
            BagFamily::None => ObjectType::Item,
            _ => ObjectType::Container,
        };

        Object {
            update_type: Object_UpdateType::CreateObject {
                guid3: self.guid,
                mask2: UpdateItemBuilder::new()
                    .set_object_guid(self.guid)
                    .set_object_entry(self.item.entry() as i32)
                    .set_object_scale_x(1.0)
                    .set_item_owner(item_owner)
                    .set_item_contained(item_owner)
                    .set_item_stack_count(self.amount as i32)
                    .set_item_durability(self.item.max_durability())
                    .set_item_maxdurability(self.item.max_durability())
                    .set_item_creator(self.creator)
                    .set_item_stack_count(self.amount as i32)
                    .finalize()
                    .into(),
                movement2: MovementBlock {
                    update_flag: MovementBlock_UpdateFlag::empty(),
                },
                object_type,
            },
        }
    }
}

pub(crate) async fn award_item(item: Item, client: &mut Client, clients: &mut [Client]) {
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

    let item_push_result = SMSG_ITEM_PUSH_RESULT {
        guid: client.character().guid,
        source: NewItemSource::Looted,
        creation_type: NewItemCreationType::Created,
        alert_chat: NewItemChatAlert::Show,
        bag_slot: 0xff,
        item_slot: item_slot.as_int() as u32,
        item: item.item.entry(),
        item_suffix_factor: 0,
        item_random_property_id: 0,
        item_count: item.amount.into(),
    };

    client.send_opcode(&item_push_result.into()).await;

    for c in clients {
        c.send_opcode(&item_push_result.into()).await;
    }
}
