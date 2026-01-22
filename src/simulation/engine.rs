// src/simulation/engine.rs

use crate::model::agent::{AgentRole, SupplyChainAgent};
use crate::model::queues::TimeDelayQueue;
use crate::simulation::config::SimulationConfig;
use crate::strategy::traits::OrderPolicy;
use serde::Serialize;

// We make this Serialize so we can write it to CSV later
#[derive(Debug, Clone, Serialize)]
pub struct HistoryRecord {
    pub week: usize,
    pub role: String,
    pub inventory: u32,
    pub backlog: u32,
    pub order_placed: u32,
    pub incoming_demand: u32,
    pub shipment_sent: u32,
    pub shipment_received: u32,
    pub cost: f32,
}

pub struct ChainSimulation {
    config: SimulationConfig,

    // The Actors
    pub agents: Vec<SupplyChainAgent>,

    // The Pipes (Delays)
    // Order Queues: Flow UPSTREAM (Retailer -> Wholesaler)
    pub order_queues: Vec<TimeDelayQueue>,
    // Shipment Queues: Flow DOWNSTREAM (Wholesaler -> Retailer)
    pub shipment_queues: Vec<TimeDelayQueue>,

    // Specific delay for Manufacturer creating goods
    pub production_delay: TimeDelayQueue,

    // Inputs/Outputs
    pub demand_schedule: Vec<u32>,
    pub current_week: usize,
    pub history: Vec<HistoryRecord>,
}

impl ChainSimulation {
    pub fn new(
        config: SimulationConfig,
        demand_schedule: Vec<u32>,
        strategies: Vec<Box<dyn OrderPolicy>>,
    ) -> Self {
        if strategies.len() != 4 {
            panic!("Must provide exactly 4 strategies.");
        }

        // Initialize Agents
        let roles = vec![
            AgentRole::Retailer,
            AgentRole::Wholesaler,
            AgentRole::Distributor,
            AgentRole::Manufacturer,
        ];

        let mut agents = Vec::new();
        for (i, strategy) in strategies.into_iter().enumerate() {
            agents.push(SupplyChainAgent::new(
                roles[i],
                config.initial_inventory,
                strategy,
            ));
        }

        // Initialize Queues
        let mut order_queues = Vec::new();
        let mut shipment_queues = Vec::new();

        // We have 3 connections between 4 agents
        for _ in 0..3 {
            order_queues.push(TimeDelayQueue::new(config.order_delay));
            shipment_queues.push(TimeDelayQueue::new(config.shipment_delay));
        }

        let production_delay = TimeDelayQueue::new(config.shipment_delay);

        Self {
            config,
            agents,
            order_queues,
            shipment_queues,
            production_delay,
            demand_schedule,
            current_week: 1, // Usually start at week 1
            history: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        // Run until we exceed max_weeks
        while self.current_week <= self.config.max_weeks {
            self.step();
        }
    }

    fn step(&mut self) {
        let week = self.current_week;

        // =================================================================
        // PHASE 1: MORNING (Arrivals)
        // Pop items out of the queues. These were put in 'delay' weeks ago.
        // =================================================================

        // 1. External Customer Demand
        // Use get() to handle if schedule is shorter than simulation
        let customer_demand = *self.demand_schedule.get(week - 1).unwrap_or(&0);

        // 2. Incoming Orders (Flowing Upstream: 0=R->W, 1=W->D, 2=D->M)
        let w_incoming_order = self.order_queues[0].pop_arrival();
        let d_incoming_order = self.order_queues[1].pop_arrival();
        let m_incoming_order = self.order_queues[2].pop_arrival();

        // 3. Incoming Shipments (Flowing Downstream: 0=W->R, 1=D->W, 2=M->D)
        let r_arrival = self.shipment_queues[0].pop_arrival();
        let w_arrival = self.shipment_queues[1].pop_arrival();
        let d_arrival = self.shipment_queues[2].pop_arrival();

        // 4. Manufacturer Production Arrival
        let m_arrival = self.production_delay.pop_arrival();

        // =================================================================
        // PHASE 2: DAY (Processing)
        // Agents update inventory and fulfill orders.
        // =================================================================

        // 1. Receive Goods (Update Inventory)
        self.agents[0].receive_shipment(r_arrival);
        self.agents[1].receive_shipment(w_arrival);
        self.agents[2].receive_shipment(d_arrival);
        self.agents[3].receive_shipment(m_arrival);

        // 2. Fulfill Orders (Ship what we can, backlog the rest)
        // Retailer handles customer
        let _r_shipped_to_customer = self.agents[0].process_order(customer_demand);
        // Upstream agents handle orders popped in Phase 1
        let w_shipped = self.agents[1].process_order(w_incoming_order);
        let d_shipped = self.agents[2].process_order(d_incoming_order);
        let m_shipped = self.agents[3].process_order(m_incoming_order);

        // 3. Make Decisions (Calculate next order)
        let r_order = self.agents[0].make_decision();
        let w_order = self.agents[1].make_decision();
        let d_order = self.agents[2].make_decision();
        let m_order = self.agents[3].make_decision();

        // =================================================================
        // PHASE 3: EVENING (Departures)
        // Push new items into the queues.
        // =================================================================

        // Push Orders (Upstream)
        self.order_queues[0].push_departure(r_order);
        self.order_queues[1].push_departure(w_order);
        self.order_queues[2].push_departure(d_order);

        // Push Shipments (Downstream)
        self.shipment_queues[0].push_departure(w_shipped);
        self.shipment_queues[1].push_departure(d_shipped);
        self.shipment_queues[2].push_departure(m_shipped);

        // Push Manufacturer Order (into production delay)
        self.production_delay.push_departure(m_order);

        // =================================================================
        // PHASE 4: RECORD & ADVANCE
        // =================================================================
        if self.current_week % 5 == 0 {
            println!(
                "Week {}: Retailer Inv: {}, Backlog: {}, Cost: ${:.2}",
                self.current_week,
                self.agents[0].inventory,
                self.agents[0].backlog,
                self.agents[0].current_cost()
            );
        }
        self.record_history();
        self.current_week += 1;
    }

    fn record_history(&mut self) {
        for agent in &self.agents {
            self.history.push(HistoryRecord {
                week: self.current_week,
                role: format!("{:?}", agent.role),
                inventory: agent.inventory,
                backlog: agent.backlog,
                order_placed: agent.last_order_placed,
                incoming_demand: agent.last_order_received,
                shipment_sent: agent.last_shipment_sent,
                shipment_received: agent.last_shipment_received,
                cost: agent.current_cost(),
            });
        }
    }

    /// Calculate the total cost for a specific agent across all weeks
    pub fn total_cost_for_agent(&self, agent_index: usize) -> f32 {
        self.history
            .iter()
            .filter(|record| record.role == format!("{:?}", self.agents[agent_index].role))
            .map(|record| record.cost)
            .sum()
    }

    /// Calculate the total cost for the entire supply chain across all weeks
    pub fn total_supply_chain_cost(&self) -> f32 {
        self.history.iter().map(|record| record.cost).sum()
    }

    /// Calculate the cost breakdown by stage
    pub fn cost_breakdown(&self) -> Vec<(String, f32)> {
        let mut breakdown = Vec::new();
        for agent in &self.agents {
            let role_name = format!("{:?}", agent.role);
            let cost = self
                .history
                .iter()
                .filter(|record| record.role == role_name)
                .map(|record| record.cost)
                .sum();
            breakdown.push((role_name, cost));
        }
        breakdown
    }
}
