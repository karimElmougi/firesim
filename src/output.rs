use std::io::stdout;

use crate::simulation::{Simulation, SimulationStep};

use num_format::{CustomFormat, ToFormattedString};
use serde::{Serialize, Serializer};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SimulationOutput {
    year: i32,

    #[serde(serialize_with = "format_num")]
    salary: i32,

    #[serde(serialize_with = "format_num", rename = "Dividend Income")]
    dividend_income: i32,

    #[serde(serialize_with = "format_num")]
    income: i32,

    #[serde(serialize_with = "format_num", rename = "Taxable Income")]
    taxable_income: i32,

    #[serde(serialize_with = "format_num", rename = "Net Income")]
    net_income: i32,

    #[serde(serialize_with = "format_num", rename = "Cost of Living")]
    cost_of_living: i32,

    #[serde(serialize_with = "format_num", rename = "Personal RRSP Contribution")]
    personal_rrsp_contribution: i32,

    #[serde(serialize_with = "format_num", rename = "Employer RRSP Contribution")]
    contribution_to_employer_rrsp: i32,

    #[serde(serialize_with = "format_num", rename = "RRSP Contribution")]
    rrsp_contribution: i32,

    #[serde(serialize_with = "format_num", rename = "TFSA Contribution")]
    tfsa_contribution: i32,

    #[serde(serialize_with = "format_num", rename = "Unregistered Contribution")]
    unregistered_contribution: i32,

    #[serde(serialize_with = "format_num", rename = "Total Contribution")]
    total_contribution: i32,

    #[serde(serialize_with = "format_num", rename = "RRSP Assets")]
    rrsp_assets: i32,

    #[serde(serialize_with = "format_num", rename = "TFSA Assets")]
    tfsa_assets: i32,

    #[serde(serialize_with = "format_num", rename = "Unregistered Assets")]
    unregistered_assets: i32,

    #[serde(serialize_with = "format_num", rename = "Total Assets")]
    total_assets: i32,

    #[serde(serialize_with = "format_num")]
    goal: i32,

    #[serde(serialize_with = "format_num", rename = "Passive Income")]
    passive_income: i32,

    #[serde(serialize_with = "format_num", rename = "Retirement Cost of Living")]
    retirement_cost_of_living: i32,
}

impl From<SimulationStep> for SimulationOutput {
    fn from(step: SimulationStep) -> Self {
        Self {
            year: step.years_since_start + 1,
            salary: step.salary(),
            dividend_income: step.dividends_income(),
            income: step.income(),
            taxable_income: step.taxable_income(),
            net_income: step.net_income(),
            cost_of_living: step.cost_of_living(),
            personal_rrsp_contribution: step.personal_rrsp_contribution,
            contribution_to_employer_rrsp: step.employer_rrsp_contribution,
            rrsp_contribution: step.total_rrsp_contribution(),
            tfsa_contribution: step.tfsa_contribution(),
            unregistered_contribution: step.unregistered_contribution(),
            total_contribution: step.total_rrsp_contribution(),
            rrsp_assets: step.rrsp_assets,
            tfsa_assets: step.tfsa_assets,
            unregistered_assets: step.unregistered_assets,
            total_assets: step.total_assets(),
            goal: step.goal(),
            passive_income: step.passive_income(),
            retirement_cost_of_living: step.retirement_cost_of_living(),
        }
    }
}

fn format_num<S>(n: &i32, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    lazy_static! {
        static ref NUM_FMT: CustomFormat = CustomFormat::builder().separator("_").build().unwrap();
    }

    let output = if *n >= 10_000 {
        n.to_formatted_string(&*NUM_FMT)
    } else {
        n.to_string()
    };

    s.serialize_str(&output)
}

pub fn print(sim: Simulation, number_of_years: usize, base_year: usize) {
    let mut writer = csv::Writer::from_writer(stdout());

    sim.take(number_of_years)
        .map(SimulationOutput::from)
        .map(|s| SimulationOutput {
            year: s.year + base_year as i32,
            ..s
        })
        .for_each(|s| writer.serialize(s).unwrap());
}
