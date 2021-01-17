use crate::tax;
use std::cmp::{max, min};

const RRSP_CONTRIBUTION_PERCENTAGE: f64 = 0.18;
const MAX_RRSP_CONTRIBUTION: i32 = 27830;
const MAX_TFSA_CONTRIBUTION: i32 = 6000;
const WITHDRAW_RATE: f64 = 0.04;

#[derive(Clone, Debug)]
pub struct SimulationContext {
    pub inflation: f64,
    pub salary_growth: f64,
    pub return_on_investment: f64,
    pub goal_multiplier: i32,
}

#[derive(Debug)]
pub struct SimulationStep {
    years_since_start: i32,
    salary: i32,
    dividends_income: i32,
    cost_of_living: i32,
    rrsp_contribution: i32,
    tfsa_contribution: i32,
    unregistered_contribution: i32,
    rrsp_assets: i32,
    tfsa_assets: i32,
    unregistered_assets: i32,
    retirement_income: i32,
    retirement_cost_of_living: i32,
    context: SimulationContext,
}

pub struct SimulationBuilder {
    salary: i32,
    cost_of_living: i32,
    rrsp_assets: i32,
    tfsa_assets: i32,
    unregistered_assets: i32,
    retirement_cost_of_living: i32,
    context: SimulationContext,
}

impl SimulationStep {
    pub fn next(&self) -> SimulationStep {
        let years_since_start = self.years_since_start + 1;

        let salary = mul(self.salary, self.context.salary_growth);
        let dividends_income = mul(
            self.unregistered_assets,
            self.context.return_on_investment - 1.0,
        );
        let income = salary + dividends_income;

        let rrsp_contribution = {
            let max_rrsp_contribution = scale(
                MAX_RRSP_CONTRIBUTION,
                self.context.inflation,
                years_since_start,
            );
            min(
                max_rrsp_contribution,
                mul(income, RRSP_CONTRIBUTION_PERCENTAGE),
            )
        };

        let taxable_income = income - rrsp_contribution;
        let net_income = tax::compute_net_income(taxable_income, years_since_start);

        let cost_of_living = mul(self.cost_of_living, self.context.inflation);

        let tfsa_contribution = {
            let max_tfsa_contribution = scale(
                MAX_TFSA_CONTRIBUTION,
                self.context.inflation,
                years_since_start,
            );
            max(0, min(max_tfsa_contribution, net_income - cost_of_living))
        };

        let unregistered_contribution = max(0, net_income - cost_of_living - tfsa_contribution);

        let rrsp_assets =
            mul(self.rrsp_assets, self.context.return_on_investment) + rrsp_contribution;

        let tfsa_assets =
            mul(self.tfsa_assets, self.context.return_on_investment) + tfsa_contribution;

        let unregistered_assets = mul(self.unregistered_assets, self.context.return_on_investment)
            + unregistered_contribution;

        let retirement_cost_of_living = mul(self.retirement_income, self.context.inflation);

        let retirement_income = withdraw_from(tfsa_assets)
            + tax::compute_net_income(
                withdraw_from(rrsp_assets + unregistered_assets),
                years_since_start,
            );

        SimulationStep {
            years_since_start,
            salary,
            dividends_income,
            cost_of_living,
            rrsp_contribution,
            tfsa_contribution,
            unregistered_contribution,
            rrsp_assets,
            tfsa_assets,
            unregistered_assets,
            retirement_income,
            retirement_cost_of_living,
            context: self.context.clone(),
        }
    }
}

impl SimulationBuilder {
    pub fn new(
        salary: i32,
        cost_of_living: i32,
        retirement_cost_of_living: i32,
        context: SimulationContext,
    ) -> SimulationBuilder {
        SimulationBuilder {
            salary,
            cost_of_living,
            rrsp_assets: 0,
            tfsa_assets: 0,
            unregistered_assets: 0,
            retirement_cost_of_living,
            context,
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

    pub fn build(self) -> SimulationStep {
        let dividends_income = mul(
            self.unregistered_assets,
            self.context.return_on_investment - 1.0,
        );
        let income = self.salary + dividends_income;

        let rrsp_contribution = min(
            MAX_RRSP_CONTRIBUTION,
            mul(income, RRSP_CONTRIBUTION_PERCENTAGE),
        );

        let taxable_income = income - rrsp_contribution;
        let net_income = tax::compute_net_income(taxable_income, 0);
        let tfsa_contribution = max(
            0,
            min(MAX_TFSA_CONTRIBUTION, net_income - self.cost_of_living),
        );

        let unregistered_contribution =
            max(0, net_income - self.cost_of_living - tfsa_contribution);

        let rrsp_assets = self.rrsp_assets + rrsp_contribution;
        let tfsa_assets = self.tfsa_assets + tfsa_contribution;
        let unregistered_assets = self.unregistered_assets + unregistered_contribution;

        let retirement_income = withdraw_from(tfsa_assets)
            + tax::compute_net_income(withdraw_from(rrsp_assets + unregistered_assets), 0);

        SimulationStep {
            years_since_start: 0,
            salary: self.salary,
            dividends_income: 0,
            cost_of_living: self.cost_of_living,
            rrsp_contribution,
            tfsa_contribution,
            unregistered_contribution,
            rrsp_assets,
            tfsa_assets,
            unregistered_assets,
            retirement_income,
            retirement_cost_of_living: self.retirement_cost_of_living,
            context: self.context,
        }
    }
}

fn scale(amount: i32, factor: f64, years: i32) -> i32 {
    mul(amount, factor.powi(years))
}

fn mul(amount: i32, factor: f64) -> i32 {
    (amount as f64 * factor) as i32
}

fn withdraw_from(assets: i32) -> i32 {
    mul(assets, WITHDRAW_RATE)
}
