mod io;
mod model;
mod simulation;
mod strategy;

use crate::io::demand;
use crate::io::reporting;
use crate::simulation::config::SimulationConfig;
use crate::simulation::engine::ChainSimulation;
use crate::strategy::implementations::{
    BaseStockPolicy, NaivePolicy, RandomPolicy, SmoothingPolicy, StermanHeuristic,
};
use crate::strategy::traits::OrderPolicy;
use std::env;

fn main() {
    println!("=== Beer Distribution Game Simulation in Rust ===");

    // 1. SETUP CONFIGURATION
    let config = SimulationConfig {
        max_weeks: 25,
        order_delay: 2,
        shipment_delay: 2,
        initial_inventory: 15, // Standard starting inventory
    };

    // 2. GENERATE DEMAND
    // We use the classic "Step" pattern: Demand jumps from 4 to 8 at week 5.
    // This is famous for triggering the Bullwhip Effect.
    let demand_schedule = demand::generate_classic_beer_game_demand(config.max_weeks);
    println!("Demand Schedule generated: {:?}", demand_schedule);

    // 3. DEFINE STRATEGIES (THE BRAINS)
    // We need exactly 4 strategies for the 4 stages:
    // Retailer -> Wholesaler -> Distributor -> Manufacturer

    // Scenario A: Everyone is Rational (Base Stock Policy)
    // trying to maintain inventory of 15 units.
    // let strategies: Vec<Box<dyn OrderPolicy>> = vec![
    //     Box::new(BaseStockPolicy::new(15)), // Retailer
    //     Box::new(BaseStockPolicy::new(15)), // Wholesaler
    //     Box::new(BaseStockPolicy::new(15)), // Distributor
    //     Box::new(BaseStockPolicy::new(15)), // Manufacturer
    // ];

    // Scenario B: One Rational Agent
    // the rest are acting using a heuristic
    let strategies: Vec<Box<dyn OrderPolicy>> = vec![
        Box::new(BaseStockPolicy::new(15)), // Retailer
        Box::new(NaivePolicy::new()),       // Wholesaler
        Box::new(NaivePolicy::new()),       // Distributor
        Box::new(NaivePolicy::new()),       // Manufacturer
    ];

    /* // Scenario C: Chaos (Mixed Agents)
    let strategies: Vec<Box<dyn OrderPolicy>> = vec![
        Box::new(NaivePolicy),                 // Retailer just panics
        Box::new(BaseStockPolicy::new(20)),    // Wholesaler hoards
        Box::new(RandomPolicy::new(0, 15)),    // Distributor is drunk
        Box::new(BaseStockPolicy::new(15)),    // Manufacturer is rational
    ];
    */

    // 4. INITIALIZE SIMULATION
    let mut sim = ChainSimulation::new(config, demand_schedule, strategies);

    // 5. RUN SIMULATION
    println!("Running simulation for 25 weeks...");
    sim.run();

    // 6. EXPORT RESULTS
    let output_file = "simulation_results.csv";
    match reporting::write_simulation_log(output_file, &sim.history) {
        Ok(_) => println!("Success! Data written to ./{}", output_file),
        Err(e) => eprintln!("Error writing CSV: {}", e),
    }

    // 7. PRINT COST ANALYSIS
    println!("\n=== Cost Analysis ===");
    let breakdown = sim.cost_breakdown();
    for (stage, cost) in breakdown {
        println!("{}: ${:.2}", stage, cost);
    }
    let total_cost = sim.total_supply_chain_cost();
    println!("Total Supply Chain Cost: ${:.2}", total_cost);

    println!("\nSimulation Complete.");
}
