mod simulation;
mod tax;

use simulation::{SimulationBuilder, SimulationConfig, SimulationStep};

use num_format::{CustomFormat, ToFormattedString};

const CSV_HEADERS: &str = "Year,Salary,Income,Taxable Income,Net Income,Cost of Living,\
            RRSP Contribution,TFSA Contribution,Unregistered Contribution,\
            RRSP Assets,TFSA Assets,Unregistered Assets,Total Assets,\
            Goal,Retirement Income";

struct NumberFormatter {
    format: CustomFormat,
}

impl NumberFormatter {
    fn new() -> NumberFormatter {
        NumberFormatter {
            format: CustomFormat::builder().separator("_").build().unwrap(),
        }
    }

    fn format(&self, n: i32) -> String {
        if n >= 10_000 {
            n.to_formatted_string(&self.format)
        } else {
            n.to_string()
        }
    }
}

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

    println!("{}", CSV_HEADERS);

    let formatter = NumberFormatter::new();
    simulation
        .take(40)
        .map(|s| to_csv(s, &formatter))
        .for_each(|s| println!("{}", s));
}

fn to_csv(step: SimulationStep, f: &NumberFormatter) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        step.years_since_start + 1,
        f.format(step.salary()),
        f.format(step.income()),
        f.format(step.taxable_income()),
        f.format(step.net_income()),
        f.format(step.cost_of_living()),
        f.format(step.rrsp_contribution),
        f.format(step.tfsa_contribution()),
        f.format(step.unregistered_contribution()),
        f.format(step.rrsp_assets),
        f.format(step.tfsa_assets),
        f.format(step.unregistered_assets),
        f.format(step.total_assets()),
        f.format(step.goal()),
        f.format(step.retirement_income())
    )
}
