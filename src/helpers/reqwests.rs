use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    prelude::Configurable,
    Error, HttpClient, ReadResponseExt,
};
use serde_json::{from_str, Value};

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
        let resp = client.get(link)?.text()?;

        if resp == "error no result" {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no result",
            )));
        }

        if let Ok(json) = from_str(&resp) {
            return Ok(json);
        } else if retry == MAX_RETRY {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Too many requests",
            )));
        }

        retry += 1;
    }
}

pub fn client(user_agent: &str, referrer: &str) -> Result<HttpClient, Error> {
    HttpClient::builder()
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .default_headers([("user-agent", user_agent), ("referer", referrer)])
        .build()
}
