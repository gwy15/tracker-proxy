use actix_web::{web, App as ActixApp, HttpServer};
use clap::{App as ClapApp, Arg};
use log::*;
use pretty_env_logger;

mod handlers;
mod torrent;

#[derive(Debug, Clone)]
pub struct AppState {
    proxy: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();

    let args = ClapApp::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Port to listen.")
                .multiple(false)
                .default_value("8080"),
        )
        .arg(
            Arg::with_name("proxy")
                .short("x")
                .long("proxy")
                .help("Proxy to use. e.g., \"socks5h://127.0.0.1:1080\"")
                .multiple(false)
                .required(true)
                .takes_value(true)
                .index(1),
        )
        .get_matches();

    let port: u32 = args
        .value_of("port")
        .expect("Arg port not given.")
        .parse()
        .expect("Port should be number.");
    let addr = format!("0.0.0.0:{}", port);

    let proxy = args.value_of("proxy").expect("Proxy must be given.");
    let state = AppState {
        proxy: proxy.to_string(),
    };

    debug!("Argument parsed");
    debug!("listen on address: {}", addr);

    HttpServer::new(move || {
        ActixApp::new()
            .data(state.clone())
            .default_service(web::route().to(handlers::handle))
    })
    .bind(addr)?
    .run()
    .await
}
