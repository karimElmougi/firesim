#[macro_use]
extern crate lazy_static;

mod accounting;
mod output;
mod simulation;

use simulation::Simulation;

use anyhow::{Context, Result};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "firesim")]
struct Opt {
    #[structopt(short, long, default_value = "20")]
    number_of_years: usize,

    #[structopt(short, long, default_value = "0")]
    base_year: usize,

    #[structopt(short, long, default_value = "config.toml")]
    config_file: String,
}

fn main() -> Result<()> {
    let options = Opt::from_args();

    let config_file_content = std::fs::read_to_string(&options.config_file)
        .with_context(|| format!("Couldn't open config file `{}`", options.config_file))?;

    let config = toml::from_str(&config_file_content).context("Invalid TOML in config file")?;

    let simulation = Simulation::new(&config);
    output::print(simulation, options.number_of_years, options.base_year);

    Ok(())
}
