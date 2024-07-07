use crate::fsrs::constants::*;

pub struct Card {
    pub id: usize,
    pub native: String,
    pub russian: String,

    pub due: u64, //epoch timestamp
    pub stability: f64, //in days
    pub difficulty: f64,
}

impl Card {
    pub fn new(native: &str, russian: &str) -> Self {
        Self {
            id: 0,
            native: native.to_owned(),
            russian: russian.to_owned(),
            due: 0,
            stability: 0.0,
            difficulty: 0.0,
        }
    }
    /// Updates the memory state
    pub fn schedule(&mut self, grade: Grade, time_of_review: u64) {
        let time = seconds_to_days(time_of_review - self.due);

        let difficulty = new_difficulty(self.difficulty, grade);
        let retrievability = retrievability(time, self.stability);
        let stability = new_stability(self.stability, self.difficulty, retrievability, grade);

        let interval = interval(self.stability, 0.9) as usize;

        self.stability = stability;
        self.difficulty = difficulty;

        self.due += days_to_seconds(interval);
    }
    
    /// First memory state
    pub fn initial_schedule(&mut self, grade: Grade) {
        let stability = initial_stability(grade);
        let difficulty = initial_difficulty(grade);

        let interval = interval(self.stability, 0.9) as usize;

        self.stability = stability;
        self.difficulty = difficulty;

        self.due += days_to_seconds(interval);
    }
}

fn initial_stability(grade: Grade) -> f64 {
    WEIGHTS[grade as usize - 1]
}

fn initial_difficulty(grade: Grade) -> f64 {
    WEIGHTS[4] - (grade as usize - 3) as f64 * WEIGHTS[5]
}

fn new_difficulty(difficulty: f64, grade: Grade) -> f64 {
    WEIGHTS[7] * initial_difficulty(Grade::Good) +
    (1f64 - WEIGHTS[7]) * (difficulty - WEIGHTS[6] * (grade as i32 - 3) as f64)
}

fn retrievability(time: usize, stability: f64) -> f64 {
    (1.0 + FACTOR * (time as f64 / stability)).powf(DECAY)
}

/// Interval until next review in days
fn interval(stability: f64, request_retention: f64) -> f64 {
    (stability / FACTOR) * (request_retention.powf(1.0 / DECAY) - 1.0)
}

fn new_stability(stability: f64, difficulty: f64, retrievability: f64, grade: Grade) -> f64 {
    match grade {
        Grade::Again => post_lapse_stability(stability, difficulty, retrievability),
        _ => stability_after_recall(stability, difficulty, retrievability, grade),
    }
}

fn stability_after_recall(stability: f64, difficulty: f64, retrievability: f64, grade: Grade) -> f64 {
    let factor = match grade {
        Grade::Hard => WEIGHTS[15],
        Grade::Easy => WEIGHTS[16],
        _ => 1.0,
    };

    stability * (WEIGHTS[8].exp() * (11.0 - difficulty) * stability.powf(-WEIGHTS[9]) *
    ((WEIGHTS[10] * (1.0 - retrievability)).exp() - 1.0) * factor + 1.0)
}

/// Calculates the new stability of a card that has been forgotten
fn post_lapse_stability(stability: f64, difficulty: f64, retrievability: f64) -> f64 {
    WEIGHTS[11] * difficulty.powf(-WEIGHTS[12]) *
    ((stability + 1.0).powf(WEIGHTS[13]) - 1.0) * (WEIGHTS[14] * (1.0 - retrievability)).exp()
}