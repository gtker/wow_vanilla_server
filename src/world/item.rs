use crate::world::database::WorldDatabase;
use wow_world_base::vanilla::ObjectType;
use wow_world_messages::vanilla::{
    MovementBlock, MovementBlock_UpdateFlag, Object, Object_UpdateType, UpdateItemBuilder,
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
                object_type: ObjectType::Item,
            },
        }
    }
}
