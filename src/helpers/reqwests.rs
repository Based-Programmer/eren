use std::process::exit;

use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    prelude::Configurable,
    AsyncReadResponseExt, Request, RequestExt,
};

const RED: &str = "\u{1b}[31m";
const RESET: &str = "\u{1b}[0m";
const YELLOW: &str = "\u{1b}[33m";

pub async fn get_isahc(link: &str, user_agent: &str, referrer: &str) -> Box<str> {
    Request::get(link)
        .header("user-agent", user_agent)
        .header("referer", referrer)
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .body(())
        .unwrap()
        .send_async()
        .await
        .unwrap_or_else(|err| {
            eprintln!(
                "Failed to get response from {YELLOW}{link}{RESET} with Error: {RED}{err}{RESET}"
            );
            exit(1);
        })
        .text()
        .await
        .unwrap()
        .into()
}
