use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    prelude::Configurable,
    Error, HttpClient, ReadResponseExt,
};
use serde_json::Value;

pub fn get_isahc(client: &HttpClient, link: &str) -> Result<Box<str>, Error> {
    Ok(client.get(link)?.text()?.into())
}

pub fn get_isahc_json(
    client: &HttpClient,
    link: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(client.get(link)?.json()?)
}

pub fn client(user_agent: &str, referrer: &str) -> Result<HttpClient, Error> {
    HttpClient::builder()
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .default_headers(&[("user-agent", user_agent), ("referer", referrer)])
        .build()
}
