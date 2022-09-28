const EXP_REQUIRED_FOR_LEVEL: [i32; 60] = [
    400, // Required to level from 1 to 2
    900, // Required to level from 2 to 3
    1400, 2100, 2800, 3600, 4500, 5400, 6500, 7600, 8800, 10100, 11400, 12900, 14400, 16000, 17700,
    19400, 21300, 23200, 25200, 27300, 29400, 31700, 34000, 36400, 38900, 41400, 44300, 47400,
    50800, 54500, 58600, 62800, 67100, 71600, 76100, 80800, 85700, 90700, 95800, 101000, 106300,
    111800, 117500, 123200, 129100, 135100, 141200, 147500, 153900, 160400, 167100, 173900, 180800,
    187900, 195000, 202300, 209800, 217400,
];

/// Get the current exp required to level up
///
/// Values `== 0 || >= 60` will return 0.
///
/// The client will automatically hide the exp bar at levels `>= 60`, so there is no sentinel value to do this.
pub const fn exp_required_to_level_up(level: u8) -> i32 {
    if level == 0 || level >= 60 {
        return 0;
    }

    EXP_REQUIRED_FOR_LEVEL[(level - 1) as usize]
}
