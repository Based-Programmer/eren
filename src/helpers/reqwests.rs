use std::error::Error;

use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    prelude::Configurable,
    AsyncReadResponseExt, HttpClient,
};

pub async fn get_isahc(client: &HttpClient, link: &str) -> Result<Box<str>, Box<dyn Error>> {
    Ok(client.get_async(link).await?.text().await?.into())
}

pub fn client(user_agent: &str, referrer: &str) -> Result<HttpClient, Box<dyn Error>> {
    Ok(HttpClient::builder()
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .default_headers(&[("user-agent", user_agent), ("referer", referrer)])
        .build()?)
}
