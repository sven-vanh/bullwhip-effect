// src/strategy/traits.rs

use std::fmt::Debug;

/// Defines the decision-making logic for a supply chain agent.
///
/// We require `Debug` so we can print the agent state if needed.
/// We require `Send` + `Sync` to allow parallel execution if you optimize later.
pub trait OrderPolicy: Debug + Send + Sync {
    /// Calculates how much to order from the upstream supplier.
    ///
    /// # Arguments
    /// * `inventory` - Current on-hand stock (u32).
    /// * `backlog` - Current unfilled orders (u32).
    /// * `incoming_demand` - The order received from downstream this turn (u32).
    /// * `supply_line` - Total goods ordered but not yet arrived (u32).
    fn calculate_order(
        &mut self,
        inventory: u32,
        backlog: u32,
        incoming_demand: u32,
        supply_line: u32,
    ) -> u32;
}
