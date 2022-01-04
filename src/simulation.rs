use crate::accounting::{self, FiscalYear};
use crate::accounting::{Constants, TaxBracket};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct InitialValues {
    salary: i32,
    cost_of_living: i32,
    retirement_cost_of_living: i32,

    #[serde(default)]
    rrsp_contribution_headroom: i32,

    #[serde(default)]
    rrsp_assets: i32,

    #[serde(default)]
    tfsa_assets: i32,

    #[serde(default)]
    unregistered_assets: i32,
}

#[derive(Debug, Deserialize, Clone)]
struct Rates {
    inflation: f64,
    salary_growth: f64,
    return_on_investment: f64,

    #[serde(default)]
    employer_rrsp_match: f64,

    #[serde(default = "default_salary_cap")]
    salary_cap: i32,

    #[serde(default = "default_withdraw_rate")]
    withdraw_rate: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(flatten)]
    rates: Rates,

    #[serde(flatten)]
    initial_values: InitialValues,

    #[serde(default, alias = "state_tax_brackets")]
    provincial_tax_brackets: Vec<TaxBracket>,

    #[serde(default)]
    federal_tax_brackets: Vec<TaxBracket>,
}

fn default_salary_cap() -> i32 {
    999_999
}

fn default_withdraw_rate() -> f64 {
    0.04
}

pub struct Simulation<'a> {
    step: SimulationStep<'a>,
}

impl<'a> Simulation<'a> {
    pub fn new(config: &'a Config) -> Self {
        Simulation {
            step: SimulationStep::new(config),
        }
    }
}

impl<'a> Iterator for Simulation<'a> {
    type Item = SimulationStep<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.step.clone();
        self.step = self.step.next();
        Some(current)
    }
}

#[derive(Debug, Clone)]
pub struct SimulationStep<'a> {
    pub fiscal_year: FiscalYear,
    pub retirement_cost_of_living: f64,
    rates: &'a Rates,
}

impl<'a> SimulationStep<'a> {
    fn new(config: &'a Config) -> SimulationStep {
        let tax_brackets = config
            .provincial_tax_brackets
            .iter()
            .chain(config.federal_tax_brackets.iter())
            .cloned()
            .collect();

        let constants = Constants::new(tax_brackets);

        let fiscal_year = FiscalYear {
            income: config.initial_values.salary as f64,
            personal_rrsp_contribution: 0.0,
            employer_rrsp_contribution: 0.0,
            rrsp_assets: config.initial_values.rrsp_assets as f64,
            tfsa_contribution: 0.0,
            tfsa_assets: config.initial_values.tfsa_assets as f64,
            unregistered_contribution: 0.0,
            unregistered_assets: config.initial_values.unregistered_assets as f64,
            cost_of_living: config.initial_values.cost_of_living as f64,
            constants,
        };

        SimulationStep {
            fiscal_year,
            retirement_cost_of_living: config.initial_values.retirement_cost_of_living as f64,
            rates: &config.rates,
        }
    }

    fn next(&self) -> Self {
        let rates = self.rates;
        let previous = self;

        let income = f64::min(
            rates.salary_cap as f64,
            previous.fiscal_year.income * rates.salary_growth,
        );

        let rrsp_contribution_headroom =
            accounting::rrsp_contribution_headroom(&previous.fiscal_year);

        let employer_rrsp_match = accounting::rrsp_match(
            income,
            rrsp_contribution_headroom,
            rates.employer_rrsp_match,
        );

        let personal_rrsp_contribution = rrsp_contribution_headroom - 2.0 * employer_rrsp_match;
        let employer_rrsp_contribution = employer_rrsp_match * 2.0;
        let total_rrsp_contribution = personal_rrsp_contribution + employer_rrsp_contribution;

        let rrsp_assets = previous.fiscal_year.rrsp_assets
            + accounting::return_on_investment(
                previous.fiscal_year.rrsp_assets,
                rates.return_on_investment,
            )
            + total_rrsp_contribution;

        let constants = previous
            .fiscal_year
            .constants
            .adjust_for_inflation(rates.inflation);

        let taxable_income = income - personal_rrsp_contribution;
        let net_income = accounting::net_income(&constants.tax_brackets, taxable_income, 0.0);

        let cost_of_living = previous.fiscal_year.cost_of_living * rates.inflation;
        let discretionary_income = net_income - cost_of_living;

        let tfsa_contribution = f64::max(
            0.0,
            f64::min(constants.tfsa_contribution_limit, discretionary_income),
        );

        let tfsa_assets = previous.fiscal_year.tfsa_assets
            + accounting::return_on_investment(
                previous.fiscal_year.tfsa_assets,
                rates.return_on_investment,
            )
            + tfsa_contribution;

        let unregistered_contribution = f64::max(0.0, discretionary_income - tfsa_contribution);

        let unregistered_assets = previous.fiscal_year.unregistered_assets
            + accounting::return_on_investment(
                previous.fiscal_year.unregistered_assets,
                rates.return_on_investment,
            )
            + unregistered_contribution;

        let next_year = FiscalYear {
            income,
            personal_rrsp_contribution,
            employer_rrsp_contribution,
            rrsp_assets,
            tfsa_contribution,
            tfsa_assets,
            unregistered_contribution,
            unregistered_assets,
            cost_of_living,
            constants,
        };

        SimulationStep {
            fiscal_year: next_year,
            retirement_cost_of_living: previous.retirement_cost_of_living * rates.inflation,
            rates,
        }
    }

    pub fn passive_income(&self) -> f64 {
        let year = &self.fiscal_year;
        let withdraw_rate = self.rates.withdraw_rate;

        year.tfsa_assets * withdraw_rate
            + accounting::net_income(
                &year.constants.tax_brackets,
                year.rrsp_assets * withdraw_rate,
                year.unregistered_assets * withdraw_rate,
            )
    }
}
