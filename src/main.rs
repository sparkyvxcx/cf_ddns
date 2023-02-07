use cf_ddns::api_client::ApiClient;
use cf_ddns::configuration::load_config;
use cf_ddns::utils::{get_args, get_current_ipv6_addr};
use colored::*;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let config_file = get_args().unwrap();
    let settings = load_config(&config_file).expect("Failed to read config");

    // println!("{:#?}", settings);

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

    println!("Fetching initial AAAA record from Cloudflare API succeed!");
    println!(
        "Starting to monitoring any global ipv6 address change on interface: {}",
        interface_name.green()
    );

    loop {
        let current_ipv6_addresses = get_current_ipv6_addr(&interface_name)
            .await
            .unwrap_or(vec![])
            .into_iter()
            .map(|addr| addr.local.clone())
            .collect();

        println!(
            "  {}'s AAAA record currently pointing to: {}",
            curr_record_domain.yellow(),
            curr_record_content.blue()
        );

        let active_ipv6_addr =
            match get_active_ipv6_addr(current_ipv6_addresses, wireguard_port).await {
                Some(addr) => addr,
                None => {
                    println!(
                        "No global reachable ipv6 address on interface: {}",
                        interface_name.red()
                    );
                    println!("Waiting 30s to retry...");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

        println!(
            "  {}'s active ipv6 address: {}",
            interface_name.green(),
            active_ipv6_addr.blue()
        );

        if active_ipv6_addr != curr_record_content {
            // TODO: update the dns record
            println!(
                "  {}'s ipv6 address changed, updating dns record...",
                interface_name.green()
            );
            match api_clt
                .set_dns_record(&dns_id, "AAAA", &curr_record_domain, &active_ipv6_addr)
                .await
            {
                Ok(_) => {
                    // TODO: send notification through telegram bot webhook, upon successful AAAA record update
                    println!("  update AAAA record through Cloudflare API succeed!");
                    curr_record_content = active_ipv6_addr;
                }
                Err(e) => {
                    println!("  {}", e);
                    println!("  sleeping 15s...");
                    tokio::time::sleep(Duration::from_secs(15)).await;
                    continue;
                }
            }
        } else {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}

async fn get_active_ipv6_addr(addr_list: Vec<String>, port: u16) -> Option<String> {
    for addr in addr_list {
        // TODO: test connectivity of each ipv6 address
        // let target_addr = Ipv6Addr::from_str(&addr.local).unwrap();
        let wg_server = format!("[{}]:{}", addr, port);
        if is_reachable(&wg_server).await {
            return Some(addr);
        }
    }
    None
}

async fn is_reachable(remote_addr: &str) -> bool {
    let remote_addr = remote_addr.parse::<SocketAddr>().unwrap();
    let sock = UdpSocket::bind("[::1]:0".parse::<SocketAddr>().unwrap())
        .await
        .expect("Failed to bind to address");

    sock.connect(remote_addr).await.is_ok()
}
