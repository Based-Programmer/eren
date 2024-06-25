use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    prelude::Configurable,
    Error, HttpClient, ReadResponseExt,
};
use serde_json::Value;
use std::io::ErrorKind::ConnectionRefused;

pub fn get_isahc(client: &HttpClient, link: &str) -> Result<Box<str>, Error> {
    Ok(client.get(link)?.text()?.into())
}

pub fn get_isahc_json(
    client: &HttpClient,
    link: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    const MAX_RETRY: u8 = 3;
    let mut retry: u8 = 0;

    loop {
        if let Ok(json) = client.get(link)?.json() {
            return Ok(json);
        } else if retry == MAX_RETRY {
            return Err(Box::new(std::io::Error::new(
                ConnectionRefused,
                "Too many requests",
            )));
        }

        retry += 1;
        // println!("{retry}");
    }
}

pub fn client(user_agent: &str, referrer: &str) -> Result<HttpClient, Error> {
    HttpClient::builder()
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .default_headers([("user-agent", user_agent), ("referer", referrer)])
        .build()
}
