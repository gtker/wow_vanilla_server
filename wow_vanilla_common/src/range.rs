pub fn trace_point_2d(from_x: f32, from_y: f32, angle: f32, distance: f32) -> (f32, f32) {
    (
        from_x + (distance * angle.cos()),
        from_y + (distance * angle.sin()),
    )
}

pub fn is_within_distance(
    from_x: f32,
    from_y: f32,
    from_z: f32,
    to_x: f32,
    to_y: f32,
    to_z: f32,
    distance: f32,
) -> bool {
    distance_to_center(from_x, from_y, from_z, to_x, to_y, to_z) < distance
}

pub fn distance_to_center(
    from_x: f32,
    from_y: f32,
    from_z: f32,
    to_x: f32,
    to_y: f32,
    to_z: f32,
) -> f32 {
    let delta_x = from_x - to_x;
    let delta_y = from_y - to_y;
    let delta_z = from_z - to_z;

    ((delta_x * delta_x) + (delta_y * delta_y) + (delta_z * delta_z)).sqrt()
}

pub fn distance_to_center_2d(from_x: f32, from_y: f32, to_x: f32, to_y: f32) -> f32 {
    let delta_x = from_x - to_x;
    let delta_y = from_y - to_y;

    ((delta_x * delta_x) + (delta_y * delta_y)).sqrt()
}

pub const SAY: f32 = 25.0;
pub const EMOTE: f32 = 25.0;
pub const YELL: f32 = 300.0;

/// Maximum range that stealth can be detected.
/// Outside of this range stealth will never be detected.
/// This is valid for both players and creatures.
pub const STEALTH_DETECTION: f32 = 30.0;

pub const TRADE: f32 = 11.11;

pub const INTERACTION: f32 = 5.0;

pub const MELEE_ATTACK: f32 = 5.0;
