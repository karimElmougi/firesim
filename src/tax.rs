use std::cmp::{max, min};

type Dollar = u32;
type Year = u32;

const INFLATION_RATE: f64 = 1.02;
const BASE_YEAR: Year = 2020;

struct TaxBracket {
    lower_bound: Dollar,
    upper_bound: Dollar,
    percentage: f64,
}

impl TaxBracket {
    fn new(lower_bound: Dollar, upper_bound: Dollar, percentage: f64) -> TaxBracket {
        TaxBracket {
            lower_bound,
            upper_bound,
            percentage,
        }
    }

    pub fn compute_tax(&self, income: Dollar) -> Dollar {
        (max(
            0,
            min(income, self.upper_bound) as i32 - self.lower_bound as i32,
        ) as f64
            * self.percentage
            / 100.0) as u32
    }

    pub fn adjust_for_inflation(&self, elapsed_years: Year, inflation_rate: f64) -> TaxBracket {
        let inflation = inflation_rate.powi(elapsed_years as i32);
        TaxBracket {
            lower_bound: (self.lower_bound as f64 * inflation) as u32,
            upper_bound: (self.upper_bound as f64 * inflation) as u32,
            ..*self
        }
    }
}

pub fn compute_provincial(income: Dollar, year: Year) -> Dollar {
    let base_brackets = [
        TaxBracket::new(0, 15_728, 0.0),
        TaxBracket::new(15_728, 45_105, 15.0),
        TaxBracket::new(45_105, 90_200, 20.0),
        TaxBracket::new(90_200, 109_755, 24.0),
        TaxBracket::new(109_755, 999_999, 25.75),
    ];

    compute_taxes(income, year, &base_brackets)
}

pub fn compute_federal(income: Dollar, year: Year) -> Dollar {
    let base_brackets = [
        TaxBracket::new(0, 13_808, 0.0),
        TaxBracket::new(13_808, 49_020, 15.0),
        TaxBracket::new(49_020, 98_040, 20.5),
        TaxBracket::new(98_040, 151_978, 26.0),
        TaxBracket::new(151_978, 216_511, 29.75),
        TaxBracket::new(216_511, 999_999, 33.0),
    ];

    compute_taxes(income, year, &base_brackets)
}

fn compute_taxes(income: Dollar, year: Year, base_brackets: &[TaxBracket]) -> Dollar {
    base_brackets
        .iter()
        .map(|b| b.adjust_for_inflation(year - BASE_YEAR, INFLATION_RATE))
        .map(|b| b.compute_tax(income))
        .sum()
}
