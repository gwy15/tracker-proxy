use actix_web::{web, HttpRequest, HttpResponse, Responder};
use log::*;
use reqwest as request;

use crate::AppState;

fn convert_request(req: HttpRequest, body: web::Bytes, proxy: &str) -> request::RequestBuilder {
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

    let proxy = request::Proxy::https(proxy).expect("Failed to load proxy");
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

async fn convert_response(response: request::Response) -> HttpResponse {
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

fn internal_error(e: request::Error) -> HttpResponse {
    HttpResponse::InternalServerError().body(format!("Internal error: {}", e.to_string()))
}

pub async fn handle(
    req: HttpRequest,
    body: web::Bytes,
    data: web::Data<AppState>,
) -> impl Responder {
    let url = req.path().to_owned();
    // get path
    let request = convert_request(req, body, &data.proxy);
    match request.send().await {
        Ok(response) => {
            info!("request success: <{}> {}", response.status(), url);
            debug!("request response: {:?}", response);
            convert_response(response).await
        }
        Err(e) => {
            warn!("request error: {}, reason: {}", url, e.to_string());
            internal_error(e)
        }
    }
}
