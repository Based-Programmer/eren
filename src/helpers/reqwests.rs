use isahc::{AsyncReadResponseExt, Request, RequestExt};

pub async fn get_isahc(link: &str, user_agent: &str, referrer: &str) -> String {
    Request::get(link)
        .header("user-agent", user_agent)
        .header("referer", referrer)
        .body(())
        .unwrap()
        .send_async()
        .await
        .unwrap_or_else(|_| panic!("Failed to get response from {}", link))
        .text()
        .await
        .unwrap()
}
