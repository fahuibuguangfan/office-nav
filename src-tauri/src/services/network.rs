use crate::error::Result;
use reqwest::Client;
use std::time::Duration;

/// 发起 HTTP 请求，支持 HTTP 和自签名证书
pub async fn fetch_text(url: &str, timeout_ms: u64) -> Result<String> {
    let client = Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .danger_accept_invalid_certs(true)  // 接受自签名证书
        .build()?;

    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {} {}", resp.status().as_u16(), url).into());
    }

    Ok(resp.text().await?)
}
