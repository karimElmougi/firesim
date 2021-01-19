use std::cmp::{max, min};
use std::rc::Rc;

use serde::Deserialize;

const RRSP_CONTRIBUTION_PERCENTAGE: f64 = 0.18;
const MAX_RRSP_CONTRIBUTION: i32 = 27830;
const MAX_TFSA_CONTRIBUTION: i32 = 6000;
const WITHDRAW_RATE: f64 = 0.04;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename(deserialize = "tax_bracket"))]
struct TaxBracket {
    #[serde(default)]
    lower_bound: i32,
    #[serde(default)]
    upper_bound: i32,
    #[serde(default)]
    percentage: f64,
}

impl TaxBracket {
    pub fn compute_tax(&self, income: i32) -> i32 {
        (max(0, min(income, self.upper_bound) - self.lower_bound) as f64 * self.percentage / 100.0)
            as i32
    }

    pub fn adjust_for_inflation(&self, elapsed_years: i32, inflation_rate: f64) -> TaxBracket {
        let inflation = inflation_rate.powi(elapsed_years);
        TaxBracket {
            lower_bound: (self.lower_bound as f64 * inflation) as i32,
            upper_bound: (self.upper_bound as f64 * inflation) as i32,
            ..*self
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    inflation: f64,
    salary_growth: f64,
    return_on_investment: f64,
    goal_multiplier: i32,
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
    #[serde(default)]
    #[serde(alias = "state_tax_brackets")]
    provincial_tax_brackets: Vec<TaxBracket>,
    #[serde(default)]
    federal_tax_brackets: Vec<TaxBracket>,
}

pub struct Simulation {
    step: SimulationStep,
}

impl Simulation {
    pub fn new(config: Config) -> Simulation {
        let config = Rc::new(config);
        let mut step = SimulationStep::new(config.clone());

        step.unregistered_assets = config.unregistered_assets;
        step.tfsa_assets = config.tfsa_assets;
        step.rrsp_assets = config.rrsp_assets;
        step.rrsp_contribution = min(
            config.rrsp_contribution_headroom,
            mul(step.income(), RRSP_CONTRIBUTION_PERCENTAGE),
        );

        step.tfsa_assets += step.tfsa_contribution();
        step.unregistered_assets += step.unregistered_contribution();
        step.rrsp_assets += step.rrsp_contribution;

        Simulation { step }
    }
}

impl Iterator for Simulation {
    type Item = SimulationStep;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.step.clone();
        self.step = self.step.next();
        Some(current)
    }
}

#[derive(Debug, Clone)]
pub struct SimulationStep {
    pub years_since_start: i32,
    pub rrsp_contribution: i32,
    pub rrsp_assets: i32,
    pub tfsa_assets: i32,
    pub unregistered_assets: i32,
    config: Rc<Config>,
}

impl SimulationStep {
    fn new(config: Rc<Config>) -> SimulationStep {
        SimulationStep {
            years_since_start: 0,
            rrsp_contribution: 0,
            rrsp_assets: 0,
            tfsa_assets: 0,
            unregistered_assets: 0,
            config,
        }
    }

    fn next(&self) -> SimulationStep {
        let config = &self.config;
        let mut next_step = self.clone();

        next_step.years_since_start += 1;

        next_step.rrsp_contribution = {
            let max_rrsp_contribution = scale(
                MAX_RRSP_CONTRIBUTION,
                config.inflation,
                next_step.years_since_start,
            );
            min(
                max_rrsp_contribution,
                mul(self.income(), RRSP_CONTRIBUTION_PERCENTAGE),
            )
        };

        next_step.rrsp_assets =
            mul(self.rrsp_assets, config.return_on_investment) + next_step.rrsp_contribution;

        next_step.tfsa_assets =
            mul(self.tfsa_assets, config.return_on_investment) + next_step.tfsa_contribution();

        next_step.unregistered_assets = mul(self.unregistered_assets, config.return_on_investment)
            + next_step.unregistered_contribution();

        next_step
    }

    pub fn salary(&self) -> i32 {
        scale(
            self.config.salary,
            self.config.salary_growth,
            self.years_since_start,
        )
    }

    pub fn dividends_income(&self) -> i32 {
        mul(
            self.unregistered_assets,
            self.config.return_on_investment - 1.0,
        )
    }

    pub fn income(&self) -> i32 {
        self.salary() + self.dividends_income()
    }

    pub fn taxable_income(&self) -> i32 {
        self.income() - self.rrsp_contribution
    }

    pub fn net_income(&self) -> i32 {
        compute_net_income(
            self.config.as_ref(),
            self.taxable_income(),
            self.years_since_start,
        )
    }

    pub fn tfsa_contribution(&self) -> i32 {
        let max_tfsa_contribution = scale(
            MAX_TFSA_CONTRIBUTION,
            self.config.inflation,
            self.years_since_start,
        );
        max(
            0,
            min(
                max_tfsa_contribution,
                self.net_income() - self.cost_of_living(),
            ),
        )
    }

    pub fn unregistered_contribution(&self) -> i32 {
        max(
            0,
            self.net_income() - self.cost_of_living() - self.tfsa_contribution(),
        )
    }

    pub fn total_assets(&self) -> i32 {
        self.rrsp_assets + self.tfsa_assets + self.unregistered_assets
    }

    pub fn cost_of_living(&self) -> i32 {
        scale(
            self.config.cost_of_living,
            self.config.inflation,
            self.years_since_start,
        )
    }

    pub fn retirement_income(&self) -> i32 {
        withdraw_from(self.tfsa_assets)
            + compute_net_income(
                self.config.as_ref(),
                withdraw_from(self.rrsp_assets + self.unregistered_assets),
                self.years_since_start,
            )
    }

    pub fn retirement_cost_of_living(&self) -> i32 {
        scale(
            self.config.retirement_cost_of_living,
            self.config.inflation,
            self.years_since_start,
        )
    }

    pub fn goal(&self) -> i32 {
        scale(
            self.config.goal_multiplier * self.retirement_cost_of_living(),
            self.config.inflation,
            self.years_since_start,
        )
    }
}

fn withdraw_from(assets: i32) -> i32 {
    mul(assets, WITHDRAW_RATE)
}

fn scale(amount: i32, factor: f64, years: i32) -> i32 {
    mul(amount, factor.powi(years))
}

fn mul(amount: i32, factor: f64) -> i32 {
    (amount as f64 * factor) as i32
}

pub fn compute_net_income(config: &Config, income: i32, elapsed_years: i32) -> i32 {
    let provincial_taxes: i32 = config
        .provincial_tax_brackets
        .iter()
        .map(|b| b.adjust_for_inflation(elapsed_years, config.inflation))
        .map(|b| b.compute_tax(income))
        .sum();

    let federal_taxes: i32 = config
        .federal_tax_brackets
        .iter()
        .map(|b| b.adjust_for_inflation(elapsed_years, config.inflation))
        .map(|b| b.compute_tax(income))
        .sum();

    income - provincial_taxes - federal_taxes
}
