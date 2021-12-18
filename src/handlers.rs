use actix_web::{web, HttpRequest, HttpResponse, Responder};
use anyhow::{Context, Result};
use log::*;
use reqwest as request;

use crate::AppState;

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
    if query.len() > 0 {
        url += "?";
        url += query;
    }

    // method, url
    let mut request_builder = client.request(req.method().to_owned(), &url);
    // headers
    for (header_name, header_value) in req.headers() {
        if header_name != "Host" {
            request_builder = request_builder.header(header_name, header_value);
        }
    }
    request_builder = request_builder.header("Host", domain);
    // body
    request_builder = request_builder.body(body);

    Ok(request_builder)
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

fn internal_error(e: anyhow::Error) -> HttpResponse {
    HttpResponse::InternalServerError().body(format!("Internal error: {}", e.to_string()))
}

/// 实际逻辑
async fn internal_handle(
    req: HttpRequest,
    body: web::Bytes,
    config: web::Data<AppState>,
) -> Result<HttpResponse> {
    let path = req.path().to_owned();
    let (domain, path) = path
        .split_once('/')
        .context("Failed to split path and locate domain.")?;

    // get path
    let request = convert_request(&domain, req, body, &config.proxy)?;
    let ret = request.send().await?;
    let ret = convert_response(ret).await;

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
            debug!("request response: {:?}", response);
            response
        }
        Err(e) => {
            warn!("request error: {:?}", e);
            internal_error(e)
        }
    }
}
