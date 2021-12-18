use actix_web::{web, HttpRequest, HttpResponse, Responder};
use anyhow::{Context, Result};
use log::*;
use reqwest as request;

use crate::{req_types::ReqType, AppState};

/// 把传进来的请求转发给 tracker，这里不需要额外的处理
fn convert_request(
    domain: &str,
    req: HttpRequest,
    body: web::Bytes,
    proxy: &str,
) -> Result<request::RequestBuilder> {
    // TODO: 这里可以复用连接，后期可以优化
    let proxy = request::Proxy::https(proxy).expect("Failed to load proxy");
    let client = request::Client::builder()
        .proxy(proxy)
        .build()
        .context("Failed to build client")?;

    // 强制使用 https，这里的 path 是包含 domain 的
    let mut url = format!("https:/{}", req.path());
    let query = req.query_string();
    if !query.is_empty() {
        url += "?";
        url += query;
    }

    // method, url
    let mut request_builder = client.request(req.method().to_owned(), &url);
    // headers
    for (header_name, header_value) in req.headers() {
        if header_name != "Host" {
            debug!("set header {} => {:?}", header_name, header_value);
            request_builder = request_builder.header(header_name, header_value);
        }
    }
    request_builder = request_builder
        .header("Host", domain)
        .header("Cache-Control", "max-age=0");

    // body
    request_builder = request_builder.body(body);

    Ok(request_builder)
}

async fn convert_response(
    response: request::Response,
    req_type: ReqType,
    domain: &str,
    tracker_proxy_url: &str,
) -> Result<HttpResponse> {
    // status
    let mut builder = actix_web::dev::HttpResponseBuilder::new(response.status());
    // headers
    for (header_name, header_value) in response.headers() {
        builder.header(header_name, header_value.clone());
    }
    // 根据类型进行修改
    let http_response = req_type
        .handle_response(response, builder, domain, tracker_proxy_url)
        .await?;

    Ok(http_response)
}

fn internal_error(e: anyhow::Error) -> HttpResponse {
    HttpResponse::InternalServerError().body(format!("Internal error: {}", e.to_string()))
}

fn get_host(req: &HttpRequest, fallback_port: u16) -> String {
    req.headers()
        .iter()
        .find_map(|(k, v)| if k == "host" { v.to_str().ok() } else { None })
        .map(|host| format!("http://{}", host))
        // fallback
        .unwrap_or_else(|| format!("http://127.0.0.1:{}", fallback_port))
}

/// 实际逻辑
async fn internal_handle(
    req: HttpRequest,
    body: web::Bytes,
    config: web::Data<AppState>,
) -> Result<HttpResponse> {
    let path = req.path().to_owned();
    let (domain, path) = path
        .trim_start_matches('/')
        .split_once('/')
        .context("Failed to split path and locate domain.")?;
    let req_type = ReqType::from_path(path);
    info!("requesting {} / {} with type {:?}", domain, path, req_type);
    let tracker_proxy_url = get_host(&req, config.port);

    // get path
    let request = convert_request(domain, req, body, &config.proxy)?;
    let response = request.send().await?;

    let ret = convert_response(response, req_type, domain, &tracker_proxy_url).await?;

    info!("request success: <{}> {} {}", ret.status(), domain, path);
    Ok(ret)
}

pub async fn handle(
    req: HttpRequest,
    body: web::Bytes,
    config: web::Data<AppState>,
) -> impl Responder {
    match internal_handle(req, body, config).await {
        Ok(response) => {
            trace!("request response: {:?}", response);
            response
        }
        Err(e) => {
            warn!("request error: {:?}", e);
            internal_error(e)
        }
    }
}
