use crate::INFLATION_RATE;
use std::cmp::{max, min};

struct TaxBracket {
    lower_bound: i32,
    upper_bound: i32,
    percentage: f64,
}

impl TaxBracket {
    fn new(lower_bound: i32, upper_bound: i32, percentage: f64) -> TaxBracket {
        TaxBracket {
            lower_bound,
            upper_bound,
            percentage,
        }
    }

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

pub fn compute_net_income(income: i32, elapsed_years: i32) -> i32 {
    income - compute_provincial(income, elapsed_years) - compute_federal(income, elapsed_years)
}

fn compute_provincial(income: i32, elapsed_years: i32) -> i32 {
    let base_brackets = [
        TaxBracket::new(0, 15_728, 0.0),
        TaxBracket::new(15_728, 45_105, 15.0),
        TaxBracket::new(45_105, 90_200, 20.0),
        TaxBracket::new(90_200, 109_755, 24.0),
        TaxBracket::new(109_755, 999_999, 25.75),
    ];

    compute_taxes(income, elapsed_years, &base_brackets)
}

fn compute_federal(income: i32, elapsed_years: i32) -> i32 {
    let base_brackets = [
        TaxBracket::new(0, 13_808, 0.0),
        TaxBracket::new(13_808, 49_020, 15.0),
        TaxBracket::new(49_020, 98_040, 20.5),
        TaxBracket::new(98_040, 151_978, 26.0),
        TaxBracket::new(151_978, 216_511, 29.75),
        TaxBracket::new(216_511, 999_999, 33.0),
    ];

    compute_taxes(income, elapsed_years, &base_brackets)
}

fn compute_taxes(income: i32, elapsed_years: i32, base_brackets: &[TaxBracket]) -> i32 {
    base_brackets
        .iter()
        .map(|b| b.adjust_for_inflation(elapsed_years, INFLATION_RATE))
        .map(|b| b.compute_tax(income))
        .sum()
}
