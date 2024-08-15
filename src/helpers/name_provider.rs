pub fn provider_name<'a>(provider: u8) -> &'a str {
    match provider {
        1 => "Ak",
        2 => "Default",
        3 => "Sak",
        4 => "S-mp4",
        5 => "Luf-mp4",
        6 => "Yt-mp4",
        _ => unreachable!(),
    }
}
