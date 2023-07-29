use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper_staticfile::Static;

use log::LevelFilter;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

async fn handle_request<B>(req: Request<B>, dir: String) -> Result<Response<Body>, std::io::Error> {
    let static_ = Static::new(Path::new(dir.as_str()));
    static_.serve(req).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let addr = args
        .get(1)
        .unwrap_or(&"127.0.0.1:8081".to_string())
        .to_string();
    let addr: SocketAddr = addr.parse().expect("Invalid address format");
    info!("Listening on http://{}", &addr);
    let dir = args.get(2).unwrap_or(&".".to_string()).to_string();
    info!("Serving \"{}\"", dir);

    let make_svc = make_service_fn(move |_conn| {
        let dir = dir.clone();
        async { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, dir.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
    Ok(())
}
