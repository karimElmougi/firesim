use std::cmp::{max, min};
use std::rc::Rc;

use serde::Deserialize;

const RRSP_CONTRIBUTION_PERCENTAGE: f64 = 0.18;
const MAX_RRSP_CONTRIBUTION: i32 = 27830;
const MAX_TFSA_CONTRIBUTION: i32 = 6000;

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

    #[serde(default = "default_withdraw_rate")]
    withdraw_rate: f64,

    #[serde(default)]
    employer_rrsp_match: f64,

    #[serde(default = "default_salary_cap")]
    salary_cap: i32,

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

impl Config {
    fn growth_rate(&self) -> f64 {
        1.0 + 0.75 * (self.return_on_investment - 1.0)
    }

    fn dividends_rate(&self) -> f64 {
        1.0 + 0.25 * (self.return_on_investment - 1.0)
    }

    fn goal_multiplier(&self) -> i32 {
        (1.0 / self.withdraw_rate) as i32
    }
}

fn default_salary_cap() -> i32 {
    999_999
}

fn default_withdraw_rate() -> f64 {
    0.04
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
        step.employer_rrsp_contribution = 2 * employer_rrsp_match(config.salary, config.rrsp_contribution_headroom, config.employer_rrsp_match);
        step.personal_rrsp_contribution = config.rrsp_contribution_headroom - step.employer_rrsp_contribution;

        step.tfsa_assets += step.tfsa_contribution();
        step.unregistered_assets += step.unregistered_contribution();
        step.rrsp_assets += step.personal_rrsp_contribution;

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
    pub personal_rrsp_contribution: i32,
    pub employer_rrsp_contribution: i32,
    pub rrsp_assets: i32,
    pub tfsa_assets: i32,
    pub unregistered_assets: i32,
    config: Rc<Config>,
}

impl SimulationStep {
    fn new(config: Rc<Config>) -> SimulationStep {
        SimulationStep {
            years_since_start: 0,
            personal_rrsp_contribution: 0,
            employer_rrsp_contribution: 0,
            rrsp_assets: 0,
            tfsa_assets: 0,
            unregistered_assets: 0,
            config,
        }
    }

    fn next(&self) -> SimulationStep {
        let config = &self.config;
        let previous_year = self;

        let years_since_start = previous_year.years_since_start + 1;

        let rrsp_contribution_headroom = rrsp_contribution_headroom(previous_year.income(), years_since_start, config.inflation);
        let employer_rrsp_match = employer_rrsp_match(previous_year.salary(), rrsp_contribution_headroom, config.employer_rrsp_match);

        let personal_rrsp_contribution = rrsp_contribution_headroom - 2 * employer_rrsp_match;
        let employer_rrsp_contribution = employer_rrsp_match * 2;
        let total_rrsp_contribution = personal_rrsp_contribution + employer_rrsp_contribution;

        let rrsp_assets =
            previous_year.rrsp_assets + previous_year.rrsp_growth() + total_rrsp_contribution;

        let mut next_year = SimulationStep {
            years_since_start,
            personal_rrsp_contribution,
            employer_rrsp_contribution,
            rrsp_assets,
            ..previous_year.clone()
        };

        next_year.tfsa_assets =
            previous_year.tfsa_assets + previous_year.tfsa_growth() + next_year.tfsa_contribution();

        next_year.unregistered_assets = previous_year.unregistered_assets
            + previous_year.unregistered_growth()
            + next_year.unregistered_contribution();

        next_year
    }

    pub fn salary(&self) -> i32 {
        let scaled_salary = scale(
            self.config.salary,
            self.config.salary_growth,
            self.years_since_start,
        );
        min(self.config.salary_cap, scaled_salary)
    }

    pub fn dividends_income(&self) -> i32 {
        mul(self.unregistered_assets, self.config.dividends_rate() - 1.0)
    }

    pub fn income(&self) -> i32 {
        self.salary() + self.dividends_income()
    }

    pub fn taxable_income(&self) -> i32 {
        self.income() - self.personal_rrsp_contribution
    }

    pub fn net_income(&self) -> i32 {
        compute_net_income(
            &self.config,
            self.years_since_start,
            self.taxable_income(),
            0,
        )
    }

    pub fn total_rrsp_contribution(&self) -> i32 {
        self.personal_rrsp_contribution + self.employer_rrsp_contribution
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
        withdraw_from(&self.config, self.tfsa_assets)
            + compute_net_income(
                self.config.as_ref(),
                self.years_since_start,
                withdraw_from(&self.config, self.rrsp_assets) + self.dividends_income(),
                withdraw_from(&self.config, self.unregistered_assets),
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
            self.config.goal_multiplier() * self.retirement_cost_of_living(),
            self.config.inflation,
            self.years_since_start,
        )
    }

    fn tfsa_growth(&self) -> i32 {
        mul(self.tfsa_assets, self.config.return_on_investment - 1.0)
    }

    fn rrsp_growth(&self) -> i32 {
        mul(self.rrsp_assets, self.config.return_on_investment - 1.0)
    }

    fn unregistered_growth(&self) -> i32 {
        mul(self.unregistered_assets, self.config.growth_rate() - 1.0)
    }
}

fn withdraw_from(config: &Config, assets: i32) -> i32 {
    mul(assets, config.withdraw_rate)
}

fn scale(amount: i32, factor: f64, years: i32) -> i32 {
    mul(amount, factor.powi(years))
}

fn mul(amount: i32, factor: f64) -> i32 {
    (amount as f64 * factor) as i32
}

fn rrsp_contribution_headroom(income: i32, years_since_start: i32, inflation_rate: f64) -> i32 {
    let max_rrsp_contribution = scale(MAX_RRSP_CONTRIBUTION, inflation_rate, years_since_start);
    min(
        max_rrsp_contribution,
        mul(income, RRSP_CONTRIBUTION_PERCENTAGE),
    )
}

fn employer_rrsp_match(
    salary: i32,
    rrsp_contribution_headroom: i32,
    employer_rrsp_match_percentage: f64,
) -> i32 {
    let max_employer_rrsp_match = mul(salary, employer_rrsp_match_percentage);
    let max_contribution_to_employer_rrsp = 2 * max_employer_rrsp_match;

    let leftover_rrsp_headroom = rrsp_contribution_headroom - max_contribution_to_employer_rrsp;
    if leftover_rrsp_headroom < 0 {
        rrsp_contribution_headroom / 2
    } else {
        max_employer_rrsp_match
    }
}

pub fn compute_net_income(
    config: &Config,
    elapsed_years: i32,
    income: i32,
    capital_gains: i32,
) -> i32 {
    let provincial_taxes: i32 = config
        .provincial_tax_brackets
        .iter()
        .map(|b| b.adjust_for_inflation(elapsed_years, config.inflation))
        .map(|b| b.compute_tax(income + capital_gains / 2))
        .sum();

    let federal_taxes: i32 = config
        .federal_tax_brackets
        .iter()
        .map(|b| b.adjust_for_inflation(elapsed_years, config.inflation))
        .map(|b| b.compute_tax(income + capital_gains / 2))
        .sum();

    income + capital_gains - provincial_taxes - federal_taxes
}

