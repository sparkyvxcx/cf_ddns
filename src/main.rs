use cf_ddns::{get_current_ipv6_addr, ApiClient};

#[tokio::main]
async fn main() {
    let settings = cf_ddns::load_config().expect("Failed to read config");
    println!("{:#?}", settings);

    let request_url = format!("{}/{}/dns_records/", settings.base_url, settings.zone_id);
    let my_client = ApiClient::new(
        request_url,
        settings.auth_email,
        settings.auth_key,
        std::time::Duration::from_secs(30),
    )
    .unwrap();

    let response = my_client.get_dns_record(&settings.dns_id).await.unwrap();
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
