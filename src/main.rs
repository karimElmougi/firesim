mod simulation;
mod tax;

use simulation::{Simulation, Config, SimulationStep};

use num_format::{CustomFormat, ToFormattedString};
use structopt::StructOpt;

const CSV_HEADERS: &str = "Year,Salary,Income,Taxable Income,Net Income,Cost of Living,\
            RRSP Contribution,TFSA Contribution,Unregistered Contribution,\
            RRSP Assets,TFSA Assets,Unregistered Assets,Total Assets,\
            Goal,Retirement Income,Retirement Cost of Living";

#[derive(StructOpt)]
#[structopt(name = "firesim")]
struct Opt {
    #[structopt(short, long, default_value = "20")]
    number_of_years: usize,

    #[structopt(short, long)]
    config_file: Option<String>,
}

struct NumberFormatter {
    format: CustomFormat,
}

impl NumberFormatter {
    fn new() -> NumberFormatter {
        NumberFormatter {
            format: CustomFormat::builder().separator("_").build().unwrap(),
        }
    }

    fn format(&self, n: i32) -> String {
        if n >= 10_000 {
            n.to_formatted_string(&self.format)
        } else {
            n.to_string()
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    let config = toml::from_str::<Config>(&match opt.config_file {
        Some(config_file) => std::fs::read_to_string(config_file).expect("can't open config file"),
        None => std::fs::read_to_string("config.toml").expect("Missing config file"),
    })
    .expect("invalid TOML in config file");

    let simulation = Simulation::new(config);

    println!("{}", CSV_HEADERS);

    let formatter = NumberFormatter::new();
    simulation
        .take(opt.number_of_years)
        .map(|s| to_csv(s, &formatter))
        .for_each(|s| println!("{}", s));
}

fn to_csv(step: SimulationStep, f: &NumberFormatter) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        step.years_since_start + 1,
        f.format(step.salary()),
        f.format(step.income()),
        f.format(step.taxable_income()),
        f.format(step.net_income()),
        f.format(step.cost_of_living()),
        f.format(step.rrsp_contribution),
        f.format(step.tfsa_contribution()),
        f.format(step.unregistered_contribution()),
        f.format(step.rrsp_assets),
        f.format(step.tfsa_assets),
        f.format(step.unregistered_assets),
        f.format(step.total_assets()),
        f.format(step.goal()),
        f.format(step.retirement_income()),
        f.format(step.retirement_cost_of_living())
    )
}
