use curl::easy::Easy;
use isahc::{AsyncReadResponseExt, Request, RequestExt};

pub async fn get_isahc(link: &str, user_agent: &str, referrer: &str) -> String {
    Request::get(link)
        .header("user-agent", user_agent)
        .header("referrer", referrer)
        .body(())
        .unwrap()
        .send_async()
        .await
        .unwrap_or_else(|_| panic!("Failed to get response from {}", link))
        .text()
        .await
        .unwrap()
}

pub async fn get_curl(link: &str, user_agent: &str, referrer: &str) -> String {
    let mut easy = Easy::new();
    easy.url(link).unwrap();
    easy.useragent(user_agent).unwrap();
    easy.referer(referrer).unwrap();

    let mut buf = Vec::new();

    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();

        transfer
            .perform()
            .unwrap_or_else(|_| panic!("Failed to get response from {}", link));
    }

    String::from_utf8(buf).unwrap()
}
