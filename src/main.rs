use cf_ddns::{get_current_ipv6_addr, ApiClient};

const BASE_URL: &'static str = "https://api.cloudflare.com/client/v4/zones";
const ZONE_ID: &'static str = "023e105f4ecef8ad9ca31a8372d0c353";
const DNS_ID: &str = "372e67954025e0ba6aaa6d586b9e0b59";
const AUTH_EMAIL: &'static str = "user@example.com";
const AUTH_KEY: &'static str = "c2547eb745079dac9320b638f5e225cf483cc5cfdda41";

#[tokio::main]
async fn main() {
    let base_url = format!("{}/{}/dns_records/", BASE_URL, ZONE_ID);
    let my_client = ApiClient::new(
        base_url,
        AUTH_EMAIL.to_string(),
        AUTH_KEY.to_string(),
        std::time::Duration::from_secs(30),
    )
    .unwrap();

    let response = my_client.get_dns_record(DNS_ID).await.unwrap();
    println!(
        "{}'s current AAAA record: {}",
        response.result.name, response.result.content
    );

    /*
    my_client
        .set_dns_record(
            DNS_ID,
            "AAAA"
            "pipipi.sparkyvxcx.me",
            "2409:8a50:da12:da0:3279:1c3:6d06:226e",
        )
        .await
        .unwrap();
    */

    let addr_info = get_current_ipv6_addr("wlan0").await.unwrap();

    println!("{:#?}", addr_info);
}
