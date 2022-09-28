pub use wow_world_base::vanilla::{Class, Map, Race};

pub mod base_stats;
pub mod class;
pub mod exp;
pub mod factions;
pub mod position;
pub mod race;
pub mod range;

pub const DEFAULT_RUNNING_SPEED: f32 = 7.0;
pub const DEFAULT_RUNNING_BACKWARDS_SPEED: f32 = 4.5;
pub const DEFAULT_WALKING_SPEED: f32 = 1.0;
pub const DEFAULT_TURN_SPEED: f32 = std::f32::consts::PI;
