use clap::{App, Arg};
use std::error::Error;
use std::process::Command;

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

pub fn get_args() -> Result<String, Box<dyn Error>> {
    let matches = App::new("cf_ddns")
        .version("0.0.1")
        .author("sparkvix <sparkvix@gmail.com>")
        .about("Cloudflare Dynamic DNS")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILENAME")
                .help("configuration file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap().to_string();

    Ok(config_path)
}

pub async fn get_current_ipv6_addr(interface_name: &str) -> Result<Vec<AddrInfo>, anyhow::Error> {
    let output = Command::new("ip")
        .arg("-6")
        .arg("-j")
        .arg("addr")
        .arg("show")
        .arg("dev")
        .arg(interface_name)
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
