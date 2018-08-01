extern crate clap;
extern crate serde;
extern crate toml;
mod config;
mod git;
mod tools;
static DEFAULT_CONFIG_PATH: &str = "/etc/auto-deploy/config.toml";

fn main() {
    let config = config::get_config(DEFAULT_CONFIG_PATH);
    println!("{:#?}", config);
}
