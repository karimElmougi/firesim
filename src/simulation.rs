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

    #[serde(default, alias = "percentage")]
    rate: f64,
}

impl TaxBracket {
    pub fn compute_tax(&self, income: f64) -> f64 {
        let portion_of_income = f64::max(
            0.0,
            f64::min(income, self.upper_bound as f64) - self.lower_bound as f64,
        );
        portion_of_income * self.rate / 100.0
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
    salary: i32,
    cost_of_living: i32,
    retirement_cost_of_living: i32,

    #[serde(default = "default_withdraw_rate")]
    withdraw_rate: f64,

    #[serde(default)]
    employer_rrsp_match: f64,

    #[serde(default = "default_salary_cap")]
    salary_cap: i32,

    #[serde(default)]
    rrsp_contribution_headroom: i32,

    #[serde(default)]
    rrsp_assets: i32,

    #[serde(default)]
    tfsa_assets: i32,

    #[serde(default)]
    unregistered_assets: i32,

    #[serde(default, alias = "state_tax_brackets")]
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

    fn goal_multiplier(&self) -> f64 {
        1.0 / self.withdraw_rate
    }
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
        let mut step = SimulationStep::new(config);

        step.unregistered_assets = config.unregistered_assets as f64;
        step.tfsa_assets = config.tfsa_assets as f64;
        step.rrsp_assets = config.rrsp_assets as f64;
        step.employer_rrsp_contribution = 2.0
            * employer_rrsp_match(
                config.salary as f64,
                config.rrsp_contribution_headroom as f64,
                config.employer_rrsp_match,
            );
        step.personal_rrsp_contribution =
            config.rrsp_contribution_headroom as f64 - step.employer_rrsp_contribution;

        step.tfsa_assets += step.tfsa_contribution();
        step.unregistered_assets += step.unregistered_contribution();
        step.rrsp_assets += step.personal_rrsp_contribution;

        Simulation { step }
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
    pub years_since_start: i32,
    pub personal_rrsp_contribution: f64,
    pub employer_rrsp_contribution: f64,
    pub rrsp_assets: f64,
    pub tfsa_assets: f64,
    pub unregistered_assets: f64,
    config: &'a Config,
}

impl<'a> SimulationStep<'a> {
    fn new(config: &'a Config) -> SimulationStep {
        SimulationStep {
            years_since_start: 0,
            personal_rrsp_contribution: 0.0,
            employer_rrsp_contribution: 0.0,
            rrsp_assets: 0.0,
            tfsa_assets: 0.0,
            unregistered_assets: 0.0,
            config,
        }
    }

    fn next(&self) -> Self {
        let config = self.config;
        let previous_year = self;

        let years_since_start = previous_year.years_since_start + 1;

        let rrsp_contribution_headroom =
            rrsp_contribution_headroom(previous_year.income(), years_since_start, config.inflation);
        let employer_rrsp_match = employer_rrsp_match(
            previous_year.salary(),
            rrsp_contribution_headroom,
            config.employer_rrsp_match,
        );

        let personal_rrsp_contribution = rrsp_contribution_headroom - 2.0 * employer_rrsp_match;
        let employer_rrsp_contribution = employer_rrsp_match * 2.0;
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

    pub fn salary(&self) -> f64 {
        let salary =
            self.config.salary as f64 * self.config.salary_growth.powi(self.years_since_start);
        f64::min(self.config.salary_cap as f64, salary)
    }

    pub fn dividends_income(&self) -> f64 {
        self.unregistered_assets * (self.config.dividends_rate() - 1.0)
    }

    pub fn income(&self) -> f64 {
        self.salary() + self.dividends_income()
    }

    pub fn taxable_income(&self) -> f64 {
        self.income() - self.personal_rrsp_contribution
    }

    pub fn net_income(&self) -> f64 {
        compute_net_income(
            self.config,
            self.years_since_start,
            self.taxable_income(),
            0.0,
        )
    }

    pub fn total_rrsp_contribution(&self) -> f64 {
        self.personal_rrsp_contribution + self.employer_rrsp_contribution
    }

    pub fn tfsa_contribution(&self) -> f64 {
        let max_tfsa_contribution =
            MAX_TFSA_CONTRIBUTION as f64 * self.config.inflation.powi(self.years_since_start);
        f64::max(
            0.0,
            f64::min(
                max_tfsa_contribution,
                self.net_income() - self.cost_of_living(),
            ),
        )
    }

    pub fn unregistered_contribution(&self) -> f64 {
        f64::max(
            0.0,
            self.net_income() - self.cost_of_living() - self.tfsa_contribution(),
        )
    }

    pub fn total_assets(&self) -> f64 {
        self.rrsp_assets + self.tfsa_assets + self.unregistered_assets
    }

    pub fn cost_of_living(&self) -> f64 {
        self.config.cost_of_living as f64 * self.config.inflation.powi(self.years_since_start)
    }

    pub fn passive_income(&self) -> f64 {
        (self.tfsa_assets * self.config.withdraw_rate)
            + compute_net_income(
                self.config,
                self.years_since_start,
                (self.rrsp_assets * self.config.withdraw_rate) + self.dividends_income(),
                self.unregistered_assets * self.config.withdraw_rate,
            )
    }

    pub fn retirement_cost_of_living(&self) -> f64 {
        self.config.retirement_cost_of_living as f64
            * self.config.inflation.powi(self.years_since_start)
    }

    pub fn goal(&self) -> f64 {
        self.retirement_cost_of_living()
            * self.config.goal_multiplier()
            * self.config.inflation.powi(self.years_since_start)
    }

    fn tfsa_growth(&self) -> f64 {
        self.tfsa_assets * (self.config.return_on_investment - 1.0)
    }

    fn rrsp_growth(&self) -> f64 {
        self.rrsp_assets * (self.config.return_on_investment - 1.0)
    }

    fn unregistered_growth(&self) -> f64 {
        self.unregistered_assets * (self.config.growth_rate() - 1.0)
    }
}

fn rrsp_contribution_headroom(income: f64, years_since_start: i32, inflation_rate: f64) -> f64 {
    let max_rrsp_contribution =
        MAX_RRSP_CONTRIBUTION as f64 * inflation_rate.powi(years_since_start);
    f64::min(max_rrsp_contribution, income * RRSP_CONTRIBUTION_PERCENTAGE)
}

fn employer_rrsp_match(
    salary: f64,
    rrsp_contribution_headroom: f64,
    employer_rrsp_match_percentage: f64,
) -> f64 {
    let max_employer_rrsp_match = salary * employer_rrsp_match_percentage;
    let max_contribution_to_employer_rrsp = 2.0 * max_employer_rrsp_match;

    let leftover_rrsp_headroom = rrsp_contribution_headroom - max_contribution_to_employer_rrsp;
    if leftover_rrsp_headroom.is_sign_negative() {
        rrsp_contribution_headroom / 2.0
    } else {
        max_employer_rrsp_match
    }
}

pub fn compute_net_income(
    config: &Config,
    elapsed_years: i32,
    income: f64,
    capital_gains: f64,
) -> f64 {
    let provincial_taxes: f64 = config
        .provincial_tax_brackets
        .iter()
        .map(|b| b.adjust_for_inflation(elapsed_years, config.inflation))
        .map(|b| b.compute_tax(income + capital_gains / 2.0))
        .sum();

    let federal_taxes: f64 = config
        .federal_tax_brackets
        .iter()
        .map(|b| b.adjust_for_inflation(elapsed_years, config.inflation))
        .map(|b| b.compute_tax(income + capital_gains / 2.0))
        .sum();

    income + capital_gains - provincial_taxes - federal_taxes
}
