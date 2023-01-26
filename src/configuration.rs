use secrecy::Secret;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Settings {
    pub cf_api_v4: CloudflareSettings,
    pub interface: InterfaceSettings,
    pub wireguard: WireguardSettings,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CloudflareSettings {
    pub base_url: String,
    pub zone_id: String,
    pub dns_id: String,
    pub auth_email: Secret<String>,
    pub auth_key: Secret<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct InterfaceSettings {
    pub name: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct WireguardSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

pub fn load_config() -> Result<Settings, config::ConfigError> {
    // Initialize config reader
    // let config_path = std::path::PathBuf::from(r"/etc/cf_ddns");
    let config_path = std::env::current_dir().expect("Failed to determine current directory");
    let settings = config::Config::builder()
        .add_source(config::File::from(config_path.join("config")))
        .build()
        .unwrap();

    // settings.merge(config::File::from(base_path.join("config")).required(true))?;
    settings.try_deserialize::<Settings>()
}
