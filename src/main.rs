extern crate clap;
extern crate git2;
extern crate serde;
extern crate toml;
extern crate url;
mod config;
mod git;
static DEFAULT_CONFIG_PATH: &str = "/etc/auto-deploy/config.toml";

fn main() {
    let config = config::get_config(DEFAULT_CONFIG_PATH);
    println!("{:#?}", config);
}
