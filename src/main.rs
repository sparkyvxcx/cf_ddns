use cf_ddns::api_client::ApiClient;
use cf_ddns::configuration::load_config;
use cf_ddns::utils::get_current_ipv6_addr;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let settings = load_config().expect("Failed to read config");
    println!("{:#?}", settings);

    let request_url = format!(
        "{}/{}/dns_records/",
        settings.cf_api_v4.base_url, settings.cf_api_v4.zone_id
    );
    let my_client = ApiClient::new(
        request_url,
        settings.cf_api_v4.auth_email,
        settings.cf_api_v4.auth_key,
        std::time::Duration::from_secs(30),
    )
    .unwrap();

    /*
    let response = my_client
        .get_dns_record(&settings.cf_api_v4.dns_id)
        .await
        .unwrap();
    println!(
        "{}'s current AAAA record: {}",
        response.result.name, response.result.content
    );

    let addr_info = get_current_ipv6_addr(&settings.interface.name)
        .await
        .unwrap();

    println!("{:#?}", addr_info);
    */

    if let Err(e) = worker_loop(
        settings.cf_api_v4.dns_id,
        settings.interface.name,
        settings.wireguard.port,
        my_client,
    )
    .await
    {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn worker_loop(
    dns_id: String,
    interface_name: String,
    wireguard_port: u16,
    api_clt: ApiClient,
) -> Result<(), anyhow::Error> {
    let curr_record = api_clt.get_dns_record(&dns_id).await?;
    let curr_record_domain = curr_record.result.name;
    let mut curr_record_content = curr_record.result.content;

    loop {
        let current_ipv6_addresses = get_current_ipv6_addr(&interface_name)
            .await?
            .iter()
            .map(|addr| format!("[{}]:{}", addr.local, wireguard_port))
            .collect();
        let active_ipv6_addr = get_active_ipv6_addr(current_ipv6_addresses)
            .await
            .expect(&format!(
                "Failed to find a active ipv6 address on interface: {}",
                interface_name
            ));

        if active_ipv6_addr != curr_record_content {
            // TODO: update the dns record
            api_clt
                .set_dns_record(&dns_id, "AAAA", &curr_record_domain, &active_ipv6_addr)
                .await
                .unwrap();

            curr_record_content = active_ipv6_addr;
        } else {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}

async fn get_active_ipv6_addr(addr_list: Vec<String>) -> Option<String> {
    for addr in addr_list {
        // TODO: test connectivity of each ipv6 address
        // let target_addr = Ipv6Addr::from_str(&addr.local).unwrap();
        if is_reachable(&addr).await {
            return Some(addr);
        }
    }
    None
}

use std::net::UdpSocket;

async fn is_reachable(target: &str) -> bool {
    let socket = UdpSocket::bind("[::1]:0").expect("Failed to bind to address");
    return socket.connect(target).is_ok();
}
