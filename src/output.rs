use crate::simulation::{Simulation, SimulationStep};

use num_format::{CustomFormat, ToFormattedString};

const CSV_HEADERS: &str = "Year,Salary,Income,Taxable Income,Net Income,Cost of Living,\
            RRSP Contribution,TFSA Contribution,Unregistered Contribution,\
            RRSP Assets,TFSA Assets,Unregistered Assets,Total Assets,\
            Goal,Retirement Income,Retirement Cost of Living";

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

pub fn print(sim: Simulation, number_of_years: usize) {
    println!("{}", CSV_HEADERS);

    let formatter = NumberFormatter::new();
    sim.take(number_of_years)
        .map(|s| to_csv(s, &formatter))
        .for_each(|s| println!("{}", s));
}

fn to_csv(step: SimulationStep, f: &NumberFormatter) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
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
        f.format(step.retirement_income()),
        f.format(step.retirement_cost_of_living())
    )
}
