extern crate clap;
extern crate git2;
extern crate java_properties;
extern crate quick_xml;
extern crate serde;
extern crate toml;
extern crate url;
extern crate yaml_rust;
mod config;
mod git;
mod projects;
static DEFAULT_CONFIG_PATH: &str = "/etc/auto-deploy/config.toml";

fn main() {
    let config = config::get_config(DEFAULT_CONFIG_PATH);
    println!("{:#?}", config);
}
