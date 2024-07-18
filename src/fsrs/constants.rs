pub const WEIGHTS: [f64; 17] = [0.4, 0.6, 2.4, 5.8, 4.93, 0.94, 0.86, 0.01, 1.49, 0.14, 0.94, 2.18, 0.05, 0.34, 1.26, 0.29, 2.61];

pub const FACTOR: f64 = 19f64/81f64;
pub const DECAY: f64 = -0.5;


#[derive(Copy, Clone)]
pub enum Grade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

/// Amount of seconds since UNIX-epoch at 5am of the day
pub fn start_of_day(secs: u64) -> u64 {
    let secs = secs - secs % 86400;
    secs + 18000
}

/// Converts seconds to days
pub fn seconds_to_days(secs: u64) -> usize {
    let secs = secs -  secs % 86400;
    (secs / 86400) as usize
}

pub fn days_to_seconds(days: usize) -> u64 {
    days as u64 * 86400
}