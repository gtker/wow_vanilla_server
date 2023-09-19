use crate::world::world::client::Client;
use crate::world::world_opcode_handler::entities::{Entities, Entity};
use wow_items::vanilla::{lookup_item, lookup_item_by_name};
use wow_world_base::geometry::trace_point_2d;
use wow_world_base::shared::Guid;
use wow_world_base::vanilla::position::{position_from_str, Position};
use wow_world_base::vanilla::{Item, Map, Vector2d};

pub(crate) enum GmCommand {
    WhereAmI,
    Teleport(Position),
    SetRunSpeed(f32),
    Mark { names: Vec<String>, p: Position },
    RangeToTarget(f32),
    AddItem(&'static Item),
    MoveNpc,
    Information(Guid),
    ShouldHaveLineOfSight(Guid),
    ShouldNotHaveLineOfSight(Guid),
}

impl GmCommand {
    pub(crate) fn from_player_command(
        message: &str,
        client: &Client,
        entities: &mut Entities,
    ) -> Result<Self, String> {
        Ok(if message == "north" {
            let mut p = client.position();
            p.x += 5.0;

            Self::Teleport(p)
        } else if message == "south" {
            let mut p = client.position();
            p.x -= 5.0;

            Self::Teleport(p)
        } else if message == "east" {
            let mut p = client.position();
            p.y -= 5.0;

            Self::Teleport(p)
        } else if message == "west" {
            let mut p = client.position();
            p.y += 5.0;

            Self::Teleport(p)
        } else if message == "whereami" {
            Self::WhereAmI
        } else if let Some(target) = message.strip_prefix("info") {
            let target = target.trim();

            let target = if let Ok(target) = target.parse::<u64>() {
                Guid::new(target)
            } else if !client.character().target.is_zero() {
                client.character().target
            } else if !target.is_empty() {
                return Err("No target selected".to_string());
            } else {
                return Err(format!("Parameter '{target}' is not a valid GUID"));
            };

            Self::Information(target)
        } else if let Some(location) = message.strip_prefix("tp") {
            let location = location.trim();
            if let Some(p) = position_from_str(location) {
                Self::Teleport(p)
            } else {
                return Err(format!("Location not found: '{}'", location));
            }
        } else if let Some(locations) = message.strip_prefix("go") {
            let locations = locations.trim();

            let coordinates: Vec<&str> = locations.split_whitespace().map(|a| a.trim()).collect();

            match coordinates.as_slice() {
                [] => {
                    if !client.character().target.is_zero() {
                        if let Some(c) = entities.find_position(client.character().target) {
                            Self::Teleport(c)
                        } else {
                            return Err(format!(
                                "Unable to find target '{}'",
                                client.character().target
                            ));
                        }
                    } else {
                        return Err(
                            "Must have a target for .go command without arguments".to_string()
                        );
                    }
                }
                [name] => {
                    let name = name.to_lowercase();
                    if let Some(c) = entities.find_position(client.character().target) {
                        Self::Teleport(c)
                    } else {
                        return Err(format!("Unable to find '{}'", name));
                    }
                }
                [_, _] => return Err("Can not teleport with only x and y coordinates".to_string()),
                [x, y, z] => {
                    let x = parse_float(x, "x coordinate")?;
                    let y = parse_float(y, "y coordinate")?;
                    let z = parse_float(z, "z coordinate")?;

                    Self::Teleport(Position {
                        map: client.character().map,
                        x,
                        y,
                        z,
                        orientation: client.character().info.orientation,
                    })
                }
                [x, y, z, map] => {
                    let x = parse_float(x, "x coordinate")?;
                    let y = parse_float(y, "y coordinate")?;
                    let z = parse_float(z, "z coordinate")?;

                    let map = parse_int(map, "map")?;
                    let map = match Map::try_from(map) {
                        Ok(e) => e,
                        Err(_) => return Err(format!("{map} is not a valid map")),
                    };

                    Self::Teleport(Position {
                        map,
                        x,
                        y,
                        z,
                        orientation: client.character().info.orientation,
                    })
                }
                _ => return Err("Incorrect '.go' command: Too many arguments".to_string()),
            }
        } else if let Some(speed) = message.strip_prefix("speed") {
            let speed = speed.trim();
            let speed = parse_float(speed, "speed argument")?;

            Self::SetRunSpeed(speed)
        } else if let Some(location) = message.strip_prefix("mark") {
            let name = location.trim();

            if name.is_empty() {
                return Err(
                    ".mark a list of names separated by a comma, like '.mark Honor Hold,HH'"
                        .to_string(),
                );
            }

            let names = name.split(",").map(|a| a.trim().to_string()).collect();

            Self::Mark {
                names,
                p: client.position(),
            }
        } else if message == "range" {
            let c = client.character();
            let target = c.target;
            if target.is_zero() {
                return Err("Unable to find range: No target".to_string());
            }

            if c.target == c.guid {
                return Err("Unable to find range: You are targeting yourself".to_string());
            }

            let (position, name) = if let Some(target) = entities.find_guid(target) {
                match target {
                    Entity::Player(c) => (c.position(), c.character().name.as_str()),
                    Entity::Creature(c) => (c.position(), c.name.as_str()),
                }
            } else {
                return Err(format!(
                    "Unable to find range: Unable to find target '{}'",
                    target
                ));
            };

            if let Some(distance) = client.distance_to_position(&position) {
                Self::RangeToTarget(distance)
            } else {
                return Err(format!(
                    "Unable to find range: Target '{name}' ({target}) is on map '{}' while you are on '{}'",
                    position.map, c.map
                ));
            }
        } else if let Some(distance) = message.strip_prefix("extend") {
            let distance = if let Ok(distance) = distance.trim().parse::<f32>() {
                distance
            } else {
                5.0
            };
            let mut p = client.position();

            let (x, y) = trace_point_2d(Vector2d { x: p.x, y: p.y }, p.orientation, distance);
            p.x = x;
            p.y = y;

            Self::Teleport(p)
        } else if let Some(distance) = message.strip_prefix("float") {
            let distance = if let Ok(distance) = distance.trim().parse::<f32>() {
                distance
            } else {
                5.0
            };
            let mut p = client.position();

            p.z += distance;

            Self::Teleport(p)
        } else if let Some(entry) = message.strip_prefix("additem") {
            let entry = if let Ok(entry) = entry.trim().parse::<u32>() {
                let Some(item) = lookup_item(entry) else {
                    return Err(format!("Unable to additem: No item with id '{entry}'"));
                };
                item
            } else if let Some(item) = lookup_item_by_name(entry.trim()) {
                item
            } else {
                return Err(format!("Unable to additem: '{entry}' is not a valid entry"));
            };

            Self::AddItem(entry)
        } else if message == "move" {
            Self::MoveNpc
        } else if message == "los" {
            Self::ShouldHaveLineOfSight(client.character().target)
        } else if message == "nolos" {
            Self::ShouldNotHaveLineOfSight(client.character().target)
        } else {
            return Err(format!("Invalid GM command: {message}"));
        })
    }
}

fn parse_int(v: &str, argument_name: &str) -> Result<i32, String> {
    match v.parse::<i32>() {
        Ok(e) => Ok(e),
        Err(_) => return Err(format!("invalid {argument_name}: '{v}'")),
    }
}

fn parse_float(v: &str, argument_name: &str) -> Result<f32, String> {
    match v.parse::<f32>() {
        Ok(e) => Ok(e),
        Err(_) => return Err(format!("invalid {argument_name}: '{v}'")),
    }
}
