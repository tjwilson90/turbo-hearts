use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::Path};

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::load(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "config.json".into()),
    )
});

#[derive(Debug, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub db_path: String,
    pub external_uri: String,
    pub port: u16,
}

impl Config {
    fn load<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    }

    pub fn redirect_uri(&self) -> String {
        format!("{}/redirect", self.external_uri)
    }
}
