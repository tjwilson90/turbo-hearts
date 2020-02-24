use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{fs::File, io::BufReader};

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load());

#[derive(Debug, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub external_uri: String,
    pub port: u16,
}

impl Config {
    fn load() -> Self {
        let file = File::open("config.json").unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    }

    pub fn redirect_uri(&self) -> String {
        format!("{}/redirect", self.external_uri)
    }
}
