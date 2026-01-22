// src/io/demand.rs

use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};

/// Generates a demand schedule where every week has the exact same order amount.
/// Useful for testing stability (e.g., step-response tests).
pub fn generate_constant_demand(weeks: usize, value: u32) -> Vec<u32> {
    vec![value; weeks]
}

/// Generates a demand schedule based on a Normal (Bell Curve) distribution.
///
/// # Arguments
/// * `weeks` - Length of the simulation.
/// * `mean` - The average order size (e.g., 10.0).
/// * `std_dev` - The standard deviation (volatility) (e.g., 2.0).
pub fn generate_normal_demand(weeks: usize, mean: f64, std_dev: f64) -> Vec<u32> {
    let mut rng = thread_rng();
    let normal = Normal::new(mean, std_dev).unwrap();

    let mut schedule = Vec::with_capacity(weeks);

    for _ in 0..weeks {
        // Sample the distribution
        let val: f64 = normal.sample(&mut rng);

        // Logic to handle conversion:
        // 1. Round to nearest integer.
        // 2. Clamp negative numbers to 0 (demand cannot be negative).
        let int_val = val.round();

        if int_val < 0.0 {
            schedule.push(0);
        } else {
            schedule.push(int_val as u32);
        }
    }

    schedule
}

/// Generates a "Step" pattern (e.g., 4 weeks of 5, then 8 for the rest).
/// This is the classic scenario used in the MIT Beer Game to trigger the Bullwhip effect.
pub fn generate_classic_beer_game_demand(weeks: usize) -> Vec<u32> {
    let mut schedule = Vec::new();
    for w in 0..weeks {
        if w < 4 {
            schedule.push(4); // Initial warm-up low demand
        } else {
            schedule.push(8); // Sudden jump to 8
        }
    }
    schedule
}
