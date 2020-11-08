use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use reqwest as request;

use log::*;
use pretty_env_logger;

const PROXY: &str = "socks5h://127.0.0.1:3214";
// const PROXY: &str = "http://127.0.0.1:3214";
const ADDR: &str = "127.0.0.1:8080";

fn convert_request(req: HttpRequest, body: web::Bytes) -> request::RequestBuilder {
    let path = req.path();
    let domain = path
        .split('/')
        .skip(1)
        .next()
        .expect("Failed to get domain");
    debug!("get request with domain = {}", domain);

    let mut url = format!("https:/{}", path);
    let query = req.query_string();
    if query.len() > 0 {
        url += "?";
        url += query;
    }

    let proxy = request::Proxy::https(PROXY).expect("Failed to load proxy");
    let client = request::Client::builder()
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    // method, url
    let mut request_builder = client.request(req.method().clone(), &url);
    // headers
    for (header_name, header_value) in req.headers() {
        if header_name != "Host" {
            request_builder = request_builder.header(header_name, header_value);
        }
    }
    request_builder = request_builder.header("Host", domain);
    // body
    request_builder = request_builder.body(body);

    request_builder
}

async fn convert_response(response: request::Response) -> impl Responder {
    // status
    let mut builder = actix_web::dev::HttpResponseBuilder::new(response.status());
    // headers
    for (header_name, header_value) in response.headers() {
        builder.header(header_name, header_value.clone());
    }
    // content
    let content = response.bytes().await.expect("Error getting bytes");
    builder.body(content)
}

async fn handler(req: HttpRequest, body: web::Bytes) -> impl Responder {
    // get path
    let request = convert_request(req, body);
    match request.send().await {
        Ok(response) => {
            info!("request success: {:?}", response);
            convert_response(response).await
        }
        Err(e) => {
            warn!("request send error: {:?}", e);
            panic!(e);
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    HttpServer::new(|| App::new().default_service(web::route().to(handler)))
        .bind(ADDR)?
        .run()
        .await
}
