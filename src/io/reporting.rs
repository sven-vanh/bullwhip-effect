// src/io/reporting.rs

use crate::simulation::engine::HistoryRecord;
use std::error::Error;
use std::path::Path;

/// Writes the simulation history to a CSV file.
///
/// # Arguments
/// * `file_path` - The path to save the file (e.g., "results/run_1.csv").
/// * `data` - The vector of history records from the simulation engine.
pub fn write_simulation_log(file_path: &str, data: &[HistoryRecord]) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file_path);

    // Create a CSV writer builder
    let mut wtr = csv::Writer::from_path(path)?;

    // Serialize and write each record
    for record in data {
        wtr.serialize(record)?;
    }

    // Flush the buffer to ensure all data is written
    wtr.flush()?;

    println!(
        "Successfully exported {} rows to '{}'",
        data.len(),
        file_path
    );
    Ok(())
}
