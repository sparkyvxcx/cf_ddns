use reqwest::{Client, Error, Url};
use secrecy::{ExposeSecret, Secret};
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
