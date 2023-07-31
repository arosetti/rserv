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
    let files_path = Path::new(&dir);
    let requested_path = req.uri().path();
    let full_path = files_path.join(requested_path.trim_start_matches('/'));

    if full_path.is_dir() {
        dir_html_response(requested_path.into(), full_path).await
    } else {
        Static::new(files_path).serve(req).await
    }
}

async fn dir_html_response(
    html_path: std::path::PathBuf,
    full_path: std::path::PathBuf,
) -> Result<Response<Body>, std::io::Error> {
    let mut html = String::new();
    html.push_str("<pre>");
    if let Ok(mut entries) = tokio::fs::read_dir(&full_path).await {
        while let Some(entry) = entries.next_entry().await.unwrap() {
            if let Some(file_name) = entry.file_name().to_str() {
                let file_path = html_path.join(file_name);
                let link = format!(
                    "<a href=\"{}\">{}</a>\n",
                    file_path.to_string_lossy(),
                    file_name
                );
                html.push_str(&link);
            }
        }
    }
    html.push_str("</pre>");
    let response = Response::builder()
        .header("Content-Type", "text/html")
        .body(Body::from(html))
        .unwrap();
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let addr = args
        .get(1)
        .unwrap_or(&"0.0.0.0:8081".to_string())
        .to_string();
    let addr: SocketAddr = addr.parse().expect("Invalid address format");
    info!("Listening on http://{addr}");
    let dir = args.get(2).unwrap_or(&".".to_string()).to_string();
    info!("Hosting: \"{dir}\"");

    let make_svc = make_service_fn(move |_conn| {
        let dir = dir.clone();
        async { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, dir.clone()))) }
    });

    if let Err(e) = Server::bind(&addr).serve(make_svc).await {
        error!("Server error: {e}");
    }
    Ok(())
}
