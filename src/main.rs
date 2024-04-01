use async_trait::async_trait;
use log::info;
use pingora::services::background::background_service;
use std::{sync::Arc, time::Duration};
use structopt::StructOpt;

use pingora::server::configuration::Opt;
use pingora::server::Server;
use pingora::upstreams::peer::HttpPeer;
use pingora::Result;
use pingora_load_balancing::{health_check, selection::RoundRobin, LoadBalancer};
use pingora_proxy::{ProxyHttp, Session};

pub struct LB(Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self
            .0
            .select(b"", 256) // hash doesn't matter
            .unwrap();

        info!("upstream peer is: {:?}", upstream);

        let peer = Box::new(HttpPeer::new(upstream, false, "one.one.one.one".to_string()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut pingora_http::RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // upstream_request
            // .insert_header("Host", "one.one.one.one")
            // .unwrap();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opt = Opt::from_args();

    let mut server = Server::new(Some(opt))?;
    server.bootstrap();

    let mut upstreams = LoadBalancer::try_from_iter(["127.0.0.1:8080"/* , "127.0.0.1:8888" */])?;

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
