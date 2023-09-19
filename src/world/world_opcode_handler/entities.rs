use crate::world::world::client::Client;
use crate::world::world_opcode_handler::creature::Creature;
use wow_world_base::shared::Guid;
use wow_world_base::vanilla::position::Position;

pub(crate) enum Entity<'a> {
    Player(&'a Client),
    Creature(&'a Creature),
}

#[derive(Debug)]
pub(crate) struct Entities<'a> {
    clients: &'a mut [Client],
    creatures: &'a mut [Creature],
}

impl<'a> Entities<'a> {
    pub(crate) fn new(clients: &'a mut [Client], creatures: &'a mut [Creature]) -> Self {
        Self { clients, creatures }
    }

    pub(crate) fn clients(&mut self) -> &mut [Client] {
        self.clients
    }

    pub(crate) fn creatures(&mut self) -> &mut [Creature] {
        self.creatures
    }

    pub(crate) fn find_guid(&self, guid: Guid) -> Option<Entity> {
        if let Some(c) = self.find_player(guid) {
            Some(Entity::Player(c))
        } else if let Some(c) = self.find_creature(guid) {
            Some(Entity::Creature(c))
        } else {
            None
        }
    }

    pub(crate) fn find_player(&self, guid: Guid) -> Option<&Client> {
        self.clients.iter().find(|c| c.character().guid == guid)
    }

    pub(crate) fn find_creature(&self, guid: Guid) -> Option<&Creature> {
        self.creatures.iter().find(|c| c.guid == guid)
    }

    pub(crate) fn find_position(&self, guid: Guid) -> Option<Position> {
        if let Some(c) = self.find_guid(guid) {
            Some(match c {
                Entity::Player(c) => c.position(),
                Entity::Creature(c) => c.position(),
            })
        } else {
            None
        }
    }
}
