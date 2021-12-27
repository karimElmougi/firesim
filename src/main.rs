#[macro_use]
extern crate lazy_static;

mod output;
mod simulation;

use simulation::{Config, Simulation};

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "firesim")]
struct Opt {
    #[structopt(short, long, default_value = "20")]
    number_of_years: usize,

    #[structopt(short, long, default_value = "0")]
    base_year: usize,

    #[structopt(short, long)]
    config_file: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    let config = toml::from_str::<Config>(&match opt.config_file {
        Some(config_file) => std::fs::read_to_string(config_file).expect("can't open config file"),
        None => std::fs::read_to_string("config.toml").expect("Missing config file"),
    })
    .expect("invalid TOML in config file");

    let simulation = Simulation::new(config);
    output::print(simulation, opt.number_of_years, opt.base_year);
}
