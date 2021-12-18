use anyhow::{Context, Result};
use log::*;
use regex::Regex;
use std::io::Read;

/// 请求是哪种
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReqType {
    /// 跟 tracker 沟通
    Tracker,
    /// 获取 rss 链接，内部会替换全部种子的链接
    Rss,
    /// 下载 torrent，内部会替换全部 tracker
    Torrent,
    /// 不知道啥类型，可能是不符合 nexusPHP 的命名规则
    Unknown,
}

impl ReqType {
    pub fn from_path(path: &str) -> Self {
        let path = path.trim_start_matches('/');
        log::debug!("deciding type: path = {}", path);
        if path.starts_with("torrentrss.php") {
            ReqType::Rss
        } else if path.starts_with("download.php") {
            ReqType::Torrent
        } else if path.starts_with("announce.php") {
            ReqType::Tracker
        } else {
            ReqType::Unknown
        }
    }

    pub async fn handle_response(
        &self,
        response: reqwest::Response,
        mut builder: actix_web::dev::HttpResponseBuilder,
        domain: &str,
        tracker_proxy_url: &str,
    ) -> Result<actix_web::HttpResponse> {
        match self {
            ReqType::Tracker => {
                // 目前 tracker 没有特殊处理
                Ok(builder.body(response.bytes().await?))
            }
            ReqType::Rss => {
                lazy_static::lazy_static! {
                    static ref DOWNLOAD_URL: Regex = Regex::new(r"http(s?)://[^/]+/download.php").unwrap();
                }
                let content = response.bytes().await?;
                let content = Self::parse_maybe_gzip_bytes(&content)?;
                log::debug!("rss content (before replace) = {}", content);

                log::info!("RSS 中有 {} 条目", content.matches("<item>").count());
                let content = DOWNLOAD_URL.replace_all(
                    &content,
                    format!("{}/{}/download.php", tracker_proxy_url, domain),
                );
                let content = content.as_ref();
                log::trace!("rss content = {:?}", content);

                // 还得压缩回去
                let bytes = Self::gzip(content.as_bytes())?;

                Ok(builder.body(bytes))
            }
            ReqType::Torrent => {
                debug!("修改种子");
                let torrent = response.bytes().await?;
                let new_body = crate::torrent::modify_torrent(&torrent, tracker_proxy_url)
                    .context("修改种子文件失败")?;
                Ok(builder.body(new_body))
            }
            ReqType::Unknown => Ok(builder.body(response.bytes().await?)),
        }
    }

    fn gzip(bytes: &[u8]) -> Result<Vec<u8>> {
        use std::io::Write;
        let mut writer = flate2::write::GzEncoder::new(Vec::new(), Default::default());
        writer.write_all(bytes)?;
        Ok(writer.finish()?)
    }

    fn parse_maybe_gzip_bytes(bytes: &[u8]) -> Result<String> {
        let mut iter = bytes.iter();
        if iter.next() == Some(&0x1f) && iter.next() == Some(&0x8b) {
            // gz
            let mut decoder = flate2::read::GzDecoder::new(bytes);
            let mut ret = String::new();
            decoder.read_to_string(&mut ret)?;
            Ok(ret)
        } else {
            Ok(String::from_utf8_lossy(bytes).to_string())
        }
    }
}

#[test]
fn test_domain_replace() {
    let body = r#"
    <comments>
    <![CDATA[ https://ourbits.club/details.php?id=165879&cmtpage=0#startcomments ]]>
    </comments>
    <enclosure url="https://ourbits.club/download.php?id=165879&passkey=1234&https=1" length="2405092748" type="application/x-bittorrent"/>
    <guid isPermaLink="false">1234</guid>
    <pubDate>Fri, 10 Dec 2021 20:09:57 +0800</pubDate>
    "#;
    let new_body = ReqType::Rss
        .handle_response(body.as_bytes(), "ourbits.club", "http://127.0.0.1:18145")
        .unwrap();
    let result = String::from_utf8(new_body).unwrap();
    assert_eq!(
        result,
        r#"
    <comments>
    <![CDATA[ https://ourbits.club/details.php?id=165879&cmtpage=0#startcomments ]]>
    </comments>
    <enclosure url="http://127.0.0.1:18145/ourbits.club/download.php?id=165879&passkey=1234&https=1" length="2405092748" type="application/x-bittorrent"/>
    <guid isPermaLink="false">1234</guid>
    <pubDate>Fri, 10 Dec 2021 20:09:57 +0800</pubDate>
    "#
    );
}
