// src/simulation/config.rs

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub max_weeks: usize,
    pub order_delay: usize,
    pub shipment_delay: usize,
    pub initial_inventory: u32,
    pub holding_cost: f64,
    pub backlog_cost: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_weeks: 25,
            order_delay: 2,
            shipment_delay: 2,
            initial_inventory: 15,
            holding_cost: 0.5,
            backlog_cost: 1.0,
        }
    }
}
