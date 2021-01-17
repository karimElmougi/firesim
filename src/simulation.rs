use crate::tax;
use std::cmp::{max, min};

const RRSP_CONTRIBUTION_PERCENTAGE: f64 = 0.18;
const MAX_RRSP_CONTRIBUTION: i32 = 27830;
const MAX_TFSA_CONTRIBUTION: i32 = 6000;
const WITHDRAW_RATE: f64 = 0.04;

pub struct Simulation {
    step: SimulationStep,
}

impl Simulation {
    pub fn csv_headers() -> String {
        "Year,Salary,Income,Taxable Income,Net Income,Cost of Living,\
            RRSP Contribution,TFSA Contribution,Unregistered Contribution,\
            RRSP Assets,TFSA Assets,Unregistered Assets,Total Assets\
            "
        .to_string()
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
pub struct SimulationConfig {
    pub inflation: f64,
    pub salary_growth: f64,
    pub return_on_investment: f64,
    pub goal_multiplier: i32,
    pub salary: i32,
    pub cost_of_living: i32,
    pub retirement_cost_of_living: i32,
}

#[derive(Debug, Clone)]
pub struct SimulationStep {
    years_since_start: i32,
    rrsp_contribution: i32,
    rrsp_assets: i32,
    tfsa_assets: i32,
    unregistered_assets: i32,
    config: SimulationConfig,
}

impl SimulationStep {
    fn new(config: SimulationConfig) -> SimulationStep {
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
        tax::compute_net_income(self.taxable_income(), self.years_since_start)
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
            + tax::compute_net_income(
                withdraw_from(self.rrsp_assets + self.unregistered_assets),
                self.years_since_start,
            )
    }


    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.years_since_start + 1,
            self.salary(),
            self.income(),
            self.taxable_income(),
            self.net_income(),
            self.cost_of_living(),
            self.rrsp_contribution,
            self.tfsa_contribution(),
            self.unregistered_contribution(),
            self.rrsp_assets,
            self.tfsa_assets,
            self.unregistered_assets,
            self.total_assets(),
        )
    }
}

pub struct SimulationBuilder {
    rrsp_assets: i32,
    tfsa_assets: i32,
    unregistered_assets: i32,
    rrsp_contribution_headroom: i32,
    config: SimulationConfig,
}

impl SimulationBuilder {
    pub fn new(config: SimulationConfig) -> SimulationBuilder {
        SimulationBuilder {
            rrsp_assets: 0,
            tfsa_assets: 0,
            unregistered_assets: 0,
            rrsp_contribution_headroom: 0,
            config,
        }
    }

    pub fn with_rrsp_assets(self, assets: i32) -> SimulationBuilder {
        SimulationBuilder {
            rrsp_assets: assets,
            ..self
        }
    }

    pub fn with_tfsa_assets(self, assets: i32) -> SimulationBuilder {
        SimulationBuilder {
            tfsa_assets: assets,
            ..self
        }
    }

    pub fn with_unregistered_assets(self, assets: i32) -> SimulationBuilder {
        SimulationBuilder {
            unregistered_assets: assets,
            ..self
        }
    }

    pub fn with_rrsp_contribution_headroom(self, headroom: i32) -> SimulationBuilder {
        SimulationBuilder {
            rrsp_contribution_headroom: headroom,
            ..self
        }
    }

    pub fn build(self) -> Simulation {
        let mut step = SimulationStep::new(self.config.clone());

        step.unregistered_assets = self.unregistered_assets;
        step.tfsa_assets = self.tfsa_assets;
        step.rrsp_assets = self.rrsp_assets;
        step.rrsp_contribution = min(
            self.rrsp_contribution_headroom,
            mul(step.income(), RRSP_CONTRIBUTION_PERCENTAGE),
        );

        step.tfsa_assets += step.tfsa_contribution();
        step.unregistered_assets += step.unregistered_contribution();
        step.rrsp_assets += step.rrsp_contribution;

        Simulation { step }
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
