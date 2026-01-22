// src/strategy/implementations.rs

use crate::simulation::config::SimulationConfig;
use crate::strategy::optimization::optimal_base_stock;
use crate::strategy::traits::{OrderContext, OrderPolicy};
use rand::Rng;

// =========================================================================
// 1. Naive Policy (Pass-Through)
// =========================================================================

/// The "Panic" strategy. It simply orders exactly what was demanded of it.
/// It ignores inventory levels and backlogs.
#[derive(Debug, Clone)]
pub struct NaivePolicy;

impl NaivePolicy {
    pub fn new() -> Self {
        Self
    }
}

impl OrderPolicy for NaivePolicy {
    fn calculate_order(
        &mut self,
        _inventory: u32,
        _backlog: u32,
        incoming_demand: u32,
        _supply_line: u32,
        _context: &OrderContext,
    ) -> u32 {
        incoming_demand
    }
}

// =========================================================================
// 2. Random Policy
// =========================================================================

/// Orders a random amount within a specific range.
/// Useful for simulating chaotic actors or testing system stability.
#[derive(Debug, Clone)]
pub struct RandomPolicy {
    min: u32,
    max: u32,
}

impl RandomPolicy {
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }
}

impl OrderPolicy for RandomPolicy {
    fn calculate_order(
        &mut self,
        _inventory: u32,
        _backlog: u32,
        _demand: u32,
        _supply_line: u32,
        _context: &OrderContext,
    ) -> u32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(self.min..=self.max)
    }
}

// =========================================================================
// 3. Base Stock Policy (Rational / "Order-Up-To")
// =========================================================================

/// A standard rational policy used in supply chain management.
///
/// It attempts to maintain a target inventory level (safety stock).
/// Formula: Order = Demand + (TargetInventory - CurrentInventory + Backlog)
///
/// If we have too much inventory, we order 0.
#[derive(Debug, Clone)]
pub struct BaseStockPolicy {
    target_stock: i32,
}

impl BaseStockPolicy {
    pub fn new(target_stock: u32) -> Self {
        Self {
            target_stock: target_stock as i32,
        }
    }

    /// Creates a BaseStockPolicy with a target calculated from cost/demand parameters
    /// (Newsvendor Model).
    pub fn with_optimal_target(
        config: &SimulationConfig,
        avg_demand: f64,
        std_dev_demand: f64,
    ) -> Self {
        let lead_time = config.order_delay + config.shipment_delay;
        let target = optimal_base_stock(
            config.backlog_cost,
            config.holding_cost,
            avg_demand,
            std_dev_demand,
            lead_time,
        );
        Self::new(target)
    }
}

impl OrderPolicy for BaseStockPolicy {
    fn calculate_order(
        &mut self,
        inventory: u32,
        backlog: u32,
        incoming_demand: u32,
        supply_line: u32,
        _context: &OrderContext,
    ) -> u32 {
        // Convert to i32 for calculation to handle negative intermediate values
        let inv = inventory as i32;
        let bl = backlog as i32;
        let demand = incoming_demand as i32;
        let supply = supply_line as i32;

        // Calculate the "Gap" we need to fill to reach target
        // Gap = Target - (Inventory - Backlog + SupplyLine)
        // SupplyLine represents goods already on the way, so we account for them
        let net_inventory = inv - bl + supply;
        let gap = self.target_stock - net_inventory;

        // The order should cover the immediate demand + fill the gap
        // If we are overstocked (gap is negative), this reduces the order.
        let raw_order = demand + gap;

        // We cannot order negative amounts.
        if raw_order < 0 {
            0
        } else {
            raw_order as u32
        }
    }
}

// =========================================================================
// 4. Sterman Heuristic Policy
// =========================================================================
// An advanced heuristic based on Sterman's research.
// It considers both inventory gaps and supply line gaps to make ordering decisions.

#[derive(Debug, Clone)]
pub struct StermanHeuristic {
    target_inventory: i32,
    target_supply_line: i32,
    alpha: f32, // Weight for Inventory Gap (0.0 - 1.0)
    beta: f32,  // Weight for Supply Line Gap (0.0 - 1.0)
}

impl StermanHeuristic {
    /// Creates a typical "Human" agent who ignores the pipeline.
    pub fn new(target_inv: u32) -> Self {
        Self {
            target_inventory: target_inv as i32,
            target_supply_line: (target_inv / 2) as i32, // Rough guess
            alpha: 1.0,                                  // Aggressively fix inventory
            beta: 0.2, // Mostly ignore what I already ordered (The fatal flaw)
        }
    }

    /// Creates a Sterman agent with optimized target parameters.
    ///
    /// The total optimal base stock (S) is split between on-hand inventory
    /// and pipeline inventory based on expected lead time consumption.
    pub fn with_optimal_target(
        config: &SimulationConfig,
        avg_demand: f64,
        std_dev_demand: f64,
    ) -> Self {
        let lead_time = config.order_delay + config.shipment_delay;
        let total_base_stock = optimal_base_stock(
            config.backlog_cost,
            config.holding_cost,
            avg_demand,
            std_dev_demand,
            lead_time,
        );

        // Decompose Base Stock (S) into On-Hand Target and Pipeline Target
        // S = Target_Inv + Target_SupplyLine
        // Target_SupplyLine is the expected amount in the pipeline
        let pipeline_target = (avg_demand * lead_time as f64).round() as i32;

        // The remainder is the desired on-hand stock (safety stock)
        let inv_target = (total_base_stock as i32) - pipeline_target;

        Self {
            target_inventory: inv_target,
            target_supply_line: pipeline_target,
            alpha: 1.0,
            beta: 0.2,
        }
    }
}

impl OrderPolicy for StermanHeuristic {
    fn calculate_order(
        &mut self,
        inventory: u32,
        backlog: u32,
        demand: u32,
        supply_line: u32,
        _context: &OrderContext,
    ) -> u32 {
        let net_inv = (inventory as i32) - (backlog as i32);
        let sl = supply_line as i32;
        let expected_demand = demand as i32; // Simplified anchor

        // Gap 1: How short am I on stock?
        let inventory_gap = (self.target_inventory - net_inv) as f32;

        // Gap 2: How short is my pipeline?
        let supply_line_gap = (self.target_supply_line - sl) as f32;

        let order =
            (expected_demand as f32) + (self.alpha * inventory_gap) + (self.beta * supply_line_gap);

        if order < 0.0 {
            0
        } else {
            order.round() as u32
        }
    }
}

// =========================================================================
// 5. Smoothing Policy
// =========================================================================
// An advanced policy that uses exponential smoothing to forecast demand
// and adjusts orders based on both the smoothed demand and inventory position.

#[derive(Debug, Clone)]
pub struct SmoothingPolicy {
    avg_demand: f32, // Internal state: Forecasting
    gamma: f32,      // Smoothing factor (0.1 = very stable, 0.9 = reactive)
    target_stock: i32,
}

impl SmoothingPolicy {
    pub fn new(initial_demand: f32, gamma: f32, target: u32) -> Self {
        Self {
            avg_demand: initial_demand,
            gamma,
            target_stock: target as i32,
        }
    }

    /// Creates a SmoothingPolicy with an optimized target stock level.
    pub fn with_optimal_target(
        initial_demand: f32,
        gamma: f32,
        config: &SimulationConfig,
        avg_demand: f64,
        std_dev_demand: f64,
    ) -> Self {
        let lead_time = config.order_delay + config.shipment_delay;
        let target = optimal_base_stock(
            config.backlog_cost,
            config.holding_cost,
            avg_demand,
            std_dev_demand,
            lead_time,
        );
        Self::new(initial_demand, gamma, target)
    }
}

impl OrderPolicy for SmoothingPolicy {
    fn calculate_order(
        &mut self,
        inventory: u32,
        backlog: u32,
        demand: u32,
        supply_line: u32,
        _context: &OrderContext,
    ) -> u32 {
        // 1. Update Forecast (Exponential Smoothing)
        self.avg_demand = (self.gamma * demand as f32) + ((1.0 - self.gamma) * self.avg_demand);

        // 2. Determine Inventory Position
        let net_inv = (inventory as i32) - (backlog as i32);
        let position = net_inv + (supply_line as i32);

        // 3. Order based on AVERAGE demand, not current demand
        // We dampen the inventory correction by gamma as well
        let inventory_correction = (self.target_stock - position) as f32 * self.gamma;

        let order = self.avg_demand + inventory_correction;

        if order < 0.0 {
            0
        } else {
            order.round() as u32
        }
    }
}

// =========================================================================
// 6. VMI Policy (Vendor Managed Inventory)
// =========================================================================

/// VMI (Vendor Managed Inventory) policy where the supplier has visibility
/// into the downstream agent's actual inventory levels and makes replenishment
/// decisions on their behalf. This eliminates information distortion and
/// reduces the bullwhip effect.
#[derive(Debug, Clone)]
pub struct VMIPolicy {
    target_stock_downstream: i32,
    target_stock_own: i32,
}

impl VMIPolicy {
    pub fn new(target_stock: u32) -> Self {
        Self {
            target_stock_downstream: target_stock as i32,
            target_stock_own: target_stock as i32,
        }
    }

    /// Creates a VMIPolicy with optimized target stock levels.
    /// Uses the same optimal target for both own and downstream stock.
    pub fn with_optimal_target(
        config: &SimulationConfig,
        avg_demand: f64,
        std_dev_demand: f64,
    ) -> Self {
        let lead_time = config.order_delay + config.shipment_delay;
        let target = optimal_base_stock(
            config.backlog_cost,
            config.holding_cost,
            avg_demand,
            std_dev_demand,
            lead_time,
        );
        Self::new(target)
    }
}

impl OrderPolicy for VMIPolicy {
    fn calculate_order(
        &mut self,
        inventory: u32,
        backlog: u32,
        _incoming_demand: u32,
        supply_line: u32,
        context: &OrderContext,
    ) -> u32 {
        // VMI: Make decisions based on downstream's ACTUAL inventory state
        // rather than their distorted orders
        if let (Some(down_inv), Some(down_back)) =
            (context.downstream_inventory, context.downstream_backlog)
        {
            // Calculate downstream's net inventory position
            let down_net = down_inv as i32 - down_back as i32;

            // Calculate how much downstream needs to reach target
            let downstream_gap = self.target_stock_downstream - down_net;

            // Also maintain our own inventory
            let own_net = inventory as i32 - backlog as i32 + supply_line as i32;
            let own_gap = self.target_stock_own - own_net;

            // Order to fill downstream's gap plus maintain our stock
            let total_order = downstream_gap + own_gap;

            if total_order < 0 {
                0
            } else {
                total_order as u32
            }
        } else {
            // Fallback: If no VMI data available, use base stock policy
            let inv = inventory as i32;
            let bl = backlog as i32;
            let supply = supply_line as i32;

            let net_inventory = inv - bl + supply;
            let gap = self.target_stock_own - net_inventory;

            if gap < 0 {
                0
            } else {
                gap as u32
            }
        }
    }
}
