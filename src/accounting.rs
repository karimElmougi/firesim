use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename(deserialize = "tax_bracket"))]
pub struct TaxBracket {
    #[serde(default)]
    lower_bound: i32,

    #[serde(default)]
    upper_bound: i32,

    #[serde(default, alias = "percentage")]
    rate: f64,
}

impl TaxBracket {
    fn compute_tax(&self, income: f64) -> f64 {
        let portion_of_income = f64::max(
            0.0,
            f64::min(income, self.upper_bound as f64) - self.lower_bound as f64,
        );
        portion_of_income * self.rate / 100.0
    }

    pub fn adjust_for_inflation(&self, inflation_rate: f64) -> TaxBracket {
        TaxBracket {
            lower_bound: (self.lower_bound as f64 * inflation_rate) as i32,
            upper_bound: (self.upper_bound as f64 * inflation_rate) as i32,
            ..*self
        }
    }
}

#[derive(Debug, Clone)]
pub struct Constants {
    pub tfsa_contribution_limit: f64,
    pub rrsp_contribution_upper_limit: f64,
    pub tax_brackets: Vec<TaxBracket>,
}

impl Constants {
    pub fn new(tax_brackets: Vec<TaxBracket>) -> Self {
        const MAX_RRSP_CONTRIBUTION: i32 = 26500; // 2019 value
        const MAX_TFSA_CONTRIBUTION: i32 = 6000; // 2021 value

        Self {
            tfsa_contribution_limit: MAX_TFSA_CONTRIBUTION as f64,
            rrsp_contribution_upper_limit: MAX_RRSP_CONTRIBUTION as f64,
            tax_brackets,
        }
    }

    pub fn adjust_for_inflation(&self, inflation_rate: f64) -> Self {
        Self {
            tfsa_contribution_limit: self.tfsa_contribution_limit * inflation_rate,
            rrsp_contribution_upper_limit: self.rrsp_contribution_upper_limit * inflation_rate,
            tax_brackets: self
                .tax_brackets
                .iter()
                .map(|b| b.adjust_for_inflation(inflation_rate))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FiscalYear {
    pub income: f64,
    pub personal_rrsp_contribution: f64,
    pub employer_rrsp_contribution: f64,
    pub rrsp_assets: f64,
    pub tfsa_contribution: f64,
    pub tfsa_assets: f64,
    pub unregistered_contribution: f64,
    pub unregistered_assets: f64,
    pub cost_of_living: f64,
    pub constants: Constants,
}

impl FiscalYear {
    pub fn taxable_income(&self) -> f64 {
        self.income - self.personal_rrsp_contribution
    }

    pub fn net_income(&self) -> f64 {
        net_income(&self.constants.tax_brackets, self.income, 0.0)
    }

    pub fn total_rrsp_contribution(&self) -> f64 {
        self.personal_rrsp_contribution + self.employer_rrsp_contribution
    }

    pub fn total_assets(&self) -> f64 {
        self.rrsp_assets + self.tfsa_assets + self.unregistered_assets
    }

    pub fn total_contribution(&self) -> f64 {
        self.total_rrsp_contribution() + self.tfsa_contribution + self.unregistered_contribution
    }
}

pub fn net_income(tax_brackets: &[TaxBracket], income: f64, capital_gains: f64) -> f64 {
    let taxes: f64 = tax_brackets
        .iter()
        .map(|b| b.compute_tax(income + capital_gains / 2.0))
        .sum();

    income + capital_gains - taxes
}

pub fn rrsp_match(salary: f64, rrsp_contribution_headroom: f64, match_percentage: f64) -> f64 {
    let personal_contribution = salary * match_percentage;
    let max_match = personal_contribution;
    let max_contribution = personal_contribution + max_match;

    let leftover_rrsp_headroom = rrsp_contribution_headroom - max_contribution;
    if leftover_rrsp_headroom.is_sign_negative() {
        rrsp_contribution_headroom / 2.0
    } else {
        max_match
    }
}

pub fn rrsp_contribution_headroom(year: &FiscalYear) -> f64 {
    const RRSP_CONTRIBUTION_PERCENTAGE: f64 = 0.18;

    f64::min(
        year.constants.rrsp_contribution_upper_limit,
        year.income * RRSP_CONTRIBUTION_PERCENTAGE,
    )
}

pub fn return_on_investment(asset: f64, rate_of_return: f64) -> f64 {
    asset * (rate_of_return - 1.0)
}
