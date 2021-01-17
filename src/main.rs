mod simulation;
mod tax;

use simulation::{Simulation, SimulationBuilder, SimulationConfig};

fn main() {
    let config = SimulationConfig {
        inflation: 1.02,
        salary_growth: 1.05,
        return_on_investment: 1.08,
        goal_multiplier: 30,
        salary: 75_000,
        cost_of_living: 20_000,
        retirement_cost_of_living: 25_000,
    };

    let simulation = SimulationBuilder::new(config)
        .with_rrsp_contribution_headroom(10_000)
        .build();

    println!("{}", Simulation::csv_headers());
    simulation.take(2).for_each(|s| println!("{}", s.to_csv()));
}
