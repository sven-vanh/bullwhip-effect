// src/strategy/optimization.rs

/// Module for supply chain optimization calculations.
///
/// This module provides tools to calculate optimal inventory parameters
/// based on cost structures and demand characteristics (The Newsvendor Model).

/// Calculates the Critical Ratio (Target Service Level).
///
/// The critical ratio represents the probability of not stocking out
/// that balances the cost of overstocking (holding) vs understocking (backlog).
///
/// Formula: CR = BacklogCost / (BacklogCost + HoldingCost)
pub fn calculate_critical_ratio(backlog_cost: f64, holding_cost: f64) -> f64 {
    if backlog_cost + holding_cost == 0.0 {
        return 0.0;
    }
    backlog_cost / (backlog_cost + holding_cost)
}

/// Approximate Inverse Cumulative Distribution Function (Quantile function) for Standard Normal Distribution.
///
/// Based on Abramowitz and Stegun formula 26.2.23.
/// The absolute error is less than 4.5e-4.
fn inverse_normal_cdf(p: f64) -> f64 {
    // Handle edge cases
    if p >= 1.0 {
        return 5.0;
    } // Cap at reasonable sigma
    if p <= 0.0 {
        return -5.0;
    }
    if p == 0.5 {
        return 0.0;
    }

    // Formula is valid for 0 < p <= 0.5
    // If p > 0.5, we use 1-p and negate the result
    let q = if p < 0.5 { p } else { 1.0 - p };

    let t = (-2.0 * q.ln()).sqrt();

    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;

    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;

    let numerator = c0 + c1 * t + c2 * t * t;
    let denominator = 1.0 + d1 * t + d2 * t * t + d3 * t * t * t;

    let x = t - (numerator / denominator);

    if p < 0.5 {
        -x
    } else {
        x
    }
}

/// Calculates the Optimal Base Stock Level (Order-Up-To Level).
///
/// Uses the Newsvendor model adapted for continuous review (or periodic review).
///
/// # Formula
/// Target Stock = MeanDemand_during_L + Z * StdDev_during_L
///
/// Where:
/// - L (Risk Horizon) = Lead Time + Review Period
/// - Z = Z-score corresponding to the Critical Ratio
///
/// # Arguments
/// * `backlog_cost` - Cost per unit of unmet demand per period.
/// * `holding_cost` - Cost per unit held in inventory per period.
/// * `avg_period_demand` - Mean demand per period (e.g., week).
/// * `std_dev_period_demand` - Standard deviation of demand per period.
/// * `lead_time_periods` - Total delay (Orders + Shipments).
///
/// # Returns
/// The optimal base stock level as an integer.
pub fn optimal_base_stock(
    backlog_cost: f64,
    holding_cost: f64,
    avg_period_demand: f64,
    std_dev_period_demand: f64,
    lead_time_periods: usize,
) -> u32 {
    let critical_ratio = calculate_critical_ratio(backlog_cost, holding_cost);
    let z_score = inverse_normal_cdf(critical_ratio);

    // Risk Horizon = Lead Time + Review Period (1 week)
    // The inventory we order now must cover us until the NEXT order arrives.
    // If Lead Time is 4 weeks, and we order weekly, we need to cover 4+1 = 5 weeks.
    let risk_horizon = (lead_time_periods + 1) as f64;

    // Calculate Mean and StdDev over the Risk Horizon
    // Assuming i.i.d demand
    let mu_l = avg_period_demand * risk_horizon;
    let sigma_l = std_dev_period_demand * risk_horizon.sqrt();

    let target_stock = mu_l + z_score * sigma_l;

    // Ensure non-negative
    if target_stock < 0.0 {
        0
    } else {
        target_stock.round() as u32
    }
}
