mod simulation;
mod tax;

const INFLATION_RATE: f64 = 1.02;

fn main() {
    let context = simulation::SimulationContext {
        inflation: 1.02,
        salary_growth: 1.05,
        return_on_investment: 1.08,
        goal_multiplier: 30,
    };
    let simulation = simulation::SimulationBuilder::new(80000, 20000, 25000, context).build();
    let next = simulation.next();
    println!("{:?}", simulation);
    println!("{:?}", next);
}
