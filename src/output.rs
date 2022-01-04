use std::io::stdout;

use crate::simulation::{Simulation, SimulationStep};

use num_format::{CustomFormat, ToFormattedString};
use serde::{Serialize, Serializer};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SimulationOutput {
    year: i32,

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

    #[serde(serialize_with = "format_num", rename = "Passive Income")]
    passive_income: i32,

    #[serde(serialize_with = "format_num", rename = "Retirement Cost of Living")]
    retirement_cost_of_living: i32,
}

impl From<(usize, SimulationStep<'_>)> for SimulationOutput {
    fn from((year, step): (usize, SimulationStep)) -> Self {
        Self {
            year: year as i32,
            income: step.fiscal_year.income as i32,
            taxable_income: step.fiscal_year.taxable_income() as i32,
            net_income: step.fiscal_year.net_income() as i32,
            cost_of_living: step.fiscal_year.cost_of_living as i32,
            personal_rrsp_contribution: step.fiscal_year.personal_rrsp_contribution as i32,
            contribution_to_employer_rrsp: step.fiscal_year.employer_rrsp_contribution as i32,
            rrsp_contribution: step.fiscal_year.total_rrsp_contribution() as i32,
            tfsa_contribution: step.fiscal_year.tfsa_contribution as i32,
            unregistered_contribution: step.fiscal_year.unregistered_contribution as i32,
            total_contribution: step.fiscal_year.total_contribution() as i32,
            rrsp_assets: step.fiscal_year.rrsp_assets as i32,
            tfsa_assets: step.fiscal_year.tfsa_assets as i32,
            unregistered_assets: step.fiscal_year.unregistered_assets as i32,
            total_assets: step.fiscal_year.total_assets() as i32,
            passive_income: step.passive_income() as i32,
            retirement_cost_of_living: step.retirement_cost_of_living as i32,
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
        .enumerate()
        .map(SimulationOutput::from)
        .map(|s| SimulationOutput {
            year: s.year + base_year as i32,
            ..s
        })
        .for_each(|s| writer.serialize(s).unwrap());
}
