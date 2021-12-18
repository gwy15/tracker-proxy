use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_bencode::value::Value;

/// see <https://en.wikipedia.org/wiki/Torrent_file>
#[derive(Debug, Deserialize, Serialize)]
struct Torrent {
    announce: String,

    #[serde(rename = "created by", skip_serializing_if = "Option::is_none")]
    created_by: Option<Value>,

    #[serde(rename = "creation date", skip_serializing_if = "Option::is_none")]
    creation_date: Option<Value>,

    info: Value,
}

pub fn modify_torrent(content: &[u8], tracker_proxy_url: &str) -> Result<Vec<u8>> {
    let mut torrent: Torrent = serde_bencode::from_bytes(content)?;
    lazy_static::lazy_static! {
        /// 这里特意只写了一个 /，这样可以保证 tracker_proxy_url 有一个 /
        static ref REPLACE_REGEX: Regex = Regex::new(r"http(s?):/").unwrap();
    }
    torrent.announce = REPLACE_REGEX
        .replace_all(&torrent.announce, tracker_proxy_url)
        .to_string();
    log::debug!("修改后：annouce = {}", torrent.announce);
    let bytes = serde_bencode::to_bytes(&torrent)?;
    Ok(bytes)
}
