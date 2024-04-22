use pingora::services::background::background_service;
use std::time::Duration;
use structopt::StructOpt;

use pingora::server::configuration::Opt;
use pingora::server::Server;
use pingora::Result;
use pingora_load_balancing::{health_check, LoadBalancer};

use std::env;

use crate::{
    constants::{P_ADDR_FOUR, P_ADDR_ONE, P_ADDR_THREE, P_ADDR_TWO},
    lb::LB,
};

mod constants;
mod lb;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let address_one = env::var(P_ADDR_ONE).unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let address_two = env::var(P_ADDR_TWO).unwrap_or_else(|_| "127.0.0.1:8888".to_owned());
    let address_three = env::var(P_ADDR_THREE).unwrap_or_else(|_| "127.0.0.1:8800".to_owned());
    let address_four = env::var(P_ADDR_FOUR).unwrap_or_else(|_| "127.0.0.1:8088".to_owned());

    let iter = [address_one, address_two, address_three, address_four];

    let ascii_art = r#"

                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⠿⣷⡀⠀
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⢟⣧⠹⣿⡀
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⠏⠼⢿⡄⢿⡇
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⠴⠞⠉⣧⢸⣷
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣰⡟⠀⠀⠀⠀⣿⢠⡿
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣿⠁⢀⣤⣴⣶⡏⢸⡇
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⣟⣻⣿⣽⣺⣿⠃⣿⠀
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⣤⣴⣶⣷⣶⣶⣶⣤⣤⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡿⠛⠛⠋⠉⠀⣼⢰⡿⠀
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⣴⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⠷⠒⠒⠊⠉⢉⡏⣾⠃⠀
                    ⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⣿⣿⣿⡿⠟⠛⠉⠛⠿⢿⣿⣿⣿⣿⣿⣿⡿⠟⠿⣷⣦⡀⠀⠀⠀⠀⠀⠀⣸⡏⠀⠀⠀⠀⠀⣼⢠⣿⠀⠀
                    ⠀⠀⠀⠀⠀⠀⣤⣿⣿⣿⣿⡟⠁⠀⠀⠀⠀⠀⠀⠀⠙⣿⣿⣿⣿⠟⠀⠀⠀⠀⠙⢿⣄⠀⠀⠀⠀⢠⣿⠃⠃⠀⠀⠀⢠⠏⣼⠃⠀⠀
                    ⠀⠀⠀⠀⠀⣼⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⣿⣿⡿⠀⠀⠀⠀⠀⠀⠈⢿⣧⠀⠀⠀⣼⡇⠀⠀⠀⠀⠀⣾⢠⣿⠀⠀⠀
                    ⠀⠀⠀⠀⢰⣿⣿⣿⣿⡿⠀⠀⠀⢀⣀⣀⣀⡀⠀⠀⠀⠀⢻⣿⣷⠀⠀⠀⣀⣤⣤⣀⣈⣻⣇⠀⢰⡿⠁⠀⠀⠀⠀⢠⠏⣾⠇⠀⠀⠀
                    ⠀⠀⠀⠀⣿⣿⣿⣿⣿⡇⠀⠀⠘⢿⣿⣿⣿⡿⠃⢠⣴⣶⣻⣛⡛⣷⣶⣄⠻⠿⠿⠿⠋⠛⣿⣦⣼⡇⠀⠀⠀⠀⠀⣾⢀⡿⠀⠀⠀⠀
                    ⠀⠀⠀⢀⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠓⠒⠖⠒⠒⠋⠁⠀⠀⠀⠀⠀⠀⢸⣿⣿⣀⡀⠀⠀⠀⢠⠃⣾⠇⠀⠀⠀⠀
                    ⠀⠀⠀⢸⣿⣿⣿⣿⣿⣧⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢿⣿⣿⣿⣿⣷⣶⣾⣴⡿⠀⠀⠀⠀⠀
                    ⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⣦⣤⣄⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣦⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠀⠀
                    ⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣿⣿⣿⣿⣿⡉⠀⠀⠀⠀⠀⠀⠀⠀
                    ⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀
                    ⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    ⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    ⠀⣠⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    ⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣿⣿⡿⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    ⠈⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡆⢀⣀⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⣾⣿⣏⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    ⠀⠀⠀⠉⠛⠻⠿⣿⣿⣿⣿⣿⣿⡿⠛⠉⠁⠀⠉⠛⢦⡄⠀⠀⠀⠀⣀⣠⣤⣴⣾⣿⣿⠿⠿⠿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⢹⣯⣠⣴⣶⣄⣰⣠⣶⠶⠿⠖⠈⠉⠉⠉⠁⠀⠀⠈⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠁⠀⠀⠉⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
	"#;

    println!("                       Penguin doing the happy dance! 🐧");
    println!("{}", ascii_art);
    let opt = Opt::from_args();

    let mut server = Server::new(Some(opt))?;
    server.bootstrap();

    let mut upstreams = LoadBalancer::try_from_iter(iter)?;

    let hc = health_check::TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency = Some(Duration::from_secs(1));

    let background = background_service("health check", upstreams);

    let upstreams = background.task();

    let mut lb = pingora_proxy::http_proxy_service(&server.configuration, LB(upstreams));
    lb.add_tcp("0.0.0.0:6188");

    server.add_service(lb);
    server.add_service(background);
    server.run_forever();

    Ok(())
}
