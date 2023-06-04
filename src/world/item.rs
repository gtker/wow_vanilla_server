use crate::world::database::WorldDatabase;
use wow_world_messages::Guid;

#[derive(Debug, Clone)]
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
}
