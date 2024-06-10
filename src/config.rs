use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub address: String,
    pub port: u16,

    pub hinting: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "localhost".to_string(),
            port: 11434,

            hinting: true,
        }
    }
}

impl From<Config> for HashMap<String, String> {
    fn from(config: Config) -> Self {
        let mut settings_kv = HashMap::new();

        settings_kv.insert("address".to_string(), config.address);
        settings_kv.insert("port".to_string(), config.port.to_string());
        settings_kv.insert("hinting".to_string(), config.hinting.to_string());

        settings_kv
    }
}
