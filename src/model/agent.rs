use serde::Serialize;
// We assume the strategy trait is defined here.
// You will create this file in the next step.
use crate::strategy::traits::{OrderContext, OrderPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum AgentRole {
    Retailer,
    Wholesaler,
    Distributor,
    Manufacturer,
}

/// The state of a single node in the supply chain.
pub struct SupplyChainAgent {
    // Identity
    pub role: AgentRole,

    // State Variables
    pub inventory: u32,
    pub backlog: u32,
    pub supply_line: u32, // Total goods ordered but not yet arrived

    // Tracking for Analysis/Logging
    pub last_order_received: u32,    // Demand from downstream
    pub last_shipment_received: u32, // Goods from upstream
    pub last_order_placed: u32,      // Decision made by this agent
    pub last_shipment_sent: u32,     // Goods sent downstream

    // The "Brain" - interchangeable decision logic
    // We exclude this from Serialize because function pointers can't be serialized to CSV easily.
    pub policy: Box<dyn OrderPolicy>,
}

impl SupplyChainAgent {
    /// Constructor for a new Agent
    pub fn new(role: AgentRole, initial_inventory: u32, policy: Box<dyn OrderPolicy>) -> Self {
        Self {
            role,
            inventory: initial_inventory,
            backlog: 0,     // Starts fresh usually
            supply_line: 0, // No orders in transit initially
            last_order_received: 0,
            last_shipment_received: 0,
            last_order_placed: 0,
            last_shipment_sent: 0,
            policy,
        }
    }

    /// Step 1: Receive goods from the upstream supplier.
    /// This reduces the supply line as goods arrive.
    pub fn receive_shipment(&mut self, quantity: u32) {
        self.inventory += quantity;
        self.last_shipment_received = quantity;

        // Reduce supply line by the amount received (capped at 0)
        if self.supply_line >= quantity {
            self.supply_line -= quantity;
        } else {
            self.supply_line = 0;
        }
    }

    /// Step 2: Handle Incoming Orders and Manage Outgoing Shipments.
    ///
    /// Returns the quantity of goods shipped downstream.
    pub fn process_order(&mut self, incoming_order: u32) -> u32 {
        self.last_order_received = incoming_order;

        // Total obligation = New Order + Old Backlog
        let total_demand = incoming_order + self.backlog;

        let amount_to_ship: u32;

        if self.inventory >= total_demand {
            // We can fill everything
            amount_to_ship = total_demand;
            self.inventory -= total_demand;
            self.backlog = 0;
        } else {
            // We are short! Ship what we have, backlog the rest.
            amount_to_ship = self.inventory;
            self.backlog = total_demand - self.inventory;
            self.inventory = 0;
        }

        self.last_shipment_sent = amount_to_ship;
        amount_to_ship
    }

    /// Step 3: Run the AI Strategy to decide what to order from upstream.
    ///
    /// Returns the quantity to order.
    pub fn make_decision(&mut self, context: &OrderContext) -> u32 {
        // The policy looks at the state and makes a decision
        let order_qty = self.policy.calculate_order(
            self.inventory,
            self.backlog,
            self.last_order_received,
            self.supply_line,
            context,
        );

        // Increase supply line by the amount we just ordered
        self.supply_line += order_qty;

        self.last_order_placed = order_qty;
        order_qty
    }

    /// Calculates current cost for this turn.
    /// Standard Beer Game costs: $0.50 per inventory unit, $1.00 per backlog unit.
    pub fn current_cost(&self) -> f32 {
        (self.inventory as f32 * 0.5) + (self.backlog as f32 * 1.0)
    }
}
