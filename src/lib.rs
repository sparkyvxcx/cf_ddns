use reqwest::{Client, Error, Url};
use secrecy::{ExposeSecret, Secret};
use std::process::Command;
use url::ParseError;

#[derive(serde::Deserialize, Debug)]
pub struct ApiResponse {
    pub result: ApiResult,
    pub success: bool,
    pub errors: Vec<String>,
    pub messages: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ApiResult {
    pub id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub name: String,
    pub content: String,
}

pub struct ApiClient {
    request_url: Url,
    http_client: Client,
    auth_email: Secret<String>,
    auth_key: Secret<String>,
}

impl ApiClient {
    pub fn new(
        request_url: String,
        auth_email: Secret<String>,
        auth_key: Secret<String>,
        timeout: std::time::Duration,
    ) -> Result<Self, ParseError> {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        let request_url = Url::parse(&request_url)?;
        Ok(Self {
            request_url,
            http_client,
            auth_email,
            auth_key,
        })
    }

    pub async fn get_dns_record(&self, record_id: &str) -> Result<ApiResponse, Error> {
        // doc: https://api.cloudflare.com/#dns-records-for-a-zone-dns-record-details
        // implement api query here
        let url = self.request_url.join(record_id).unwrap();

        println!("Request URL: {}", url);
        let response = self
            .http_client
            .get(url)
            .header("X-Auth-Email", self.auth_email.expose_secret())
            .header("X-Auth-Key", self.auth_key.expose_secret())
            .header("Content-Type", "application/json")
            .send()
            .await?;
        // parse json response to struct here
        let response_json: ApiResponse = response.json().await?;
        println!("{:#?}", response_json);

        Ok(response_json)
    }

    pub async fn set_dns_record(
        &self,
        record_id: &str,
        record_type: &str,
        record_name: &str,
        record_content: &str,
    ) -> Result<(), Error> {
        // doc: https://api.cloudflare.com/#dns-records-for-a-zone-update-dns-record
        // implement api update here
        let url = self.request_url.join(record_id).unwrap();

        println!("Request URL: {}", url);
        let request_body = serde_json::json!({
            "type": record_type,
            "name": record_name,
            "content": record_content,
            "ttl": 1,
            "proxied": false,
            "comment": "Wireguard redirection record",
        });
        let response = self
            .http_client
            .put(url)
            .header("X-Auth-Email", self.auth_email.expose_secret())
            .header("X-Auth-Key", self.auth_key.expose_secret())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        // parse json response to struct here
        // let response_json: ApiResponse = response.json().await?;
        let response_text = response.text().await?;
        println!("{:?}", response_text);

        Ok(())
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct InterfaceInfo {
    pub ifindex: usize,
    pub ifname: String,
    pub flags: Vec<String>,
    pub mtu: usize,
    pub qdisc: String,
    pub operstate: String,
    pub group: String,
    pub txqlen: usize,
    pub addr_info: Vec<AddrInfo>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AddrInfo {
    pub family: String,
    pub local: String,
    pub prefixlen: usize,
    pub scope: String,
    //  dynamic: bool,
    //  mngtmpaddr: bool,
    //  noprefixroute: bool,
    //  valid_life_time: usize,
    //  preferred_life_time: usize,
}

pub async fn get_current_ipv6_addr(
    interface: &str,
) -> Result<Vec<AddrInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("ip")
        .arg("-6")
        .arg("-j")
        .arg("addr")
        .arg("show")
        .arg("dev")
        .arg(interface)
        .output()
        .expect("failed to execute process");

    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    let cmd_result = String::from_utf8_lossy(&output.stdout);

    let interface_info_vec: Vec<InterfaceInfo> = serde_json::from_str(&cmd_result)?;

    // println!("{:#?}", interface_info_vec[0]);

    let addr_info = interface_info_vec[0].addr_info.to_owned();
    Ok(addr_info)
}

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub base_url: String,
    pub zone_id: String,
    pub dns_id: String,
    pub auth_email: Secret<String>,
    pub auth_key: Secret<String>,
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
