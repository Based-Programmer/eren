pub fn provider_num(provider: &str) -> u8 {
    match provider {
        "Ak" => 1,
        "Default" => 2,
        "S-mp4" => 3,
        "Luf-mp4" => 4,
        "Yt-mp4" => 5,
        _ => unreachable!(),
    }
}

pub fn provider_name<'a>(provider: u8) -> &'a str {
    match provider {
        1 => "Ak",
        2 => "Default",
        3 => "S-mp4",
        4 => "Luf-mp4",
        5 => "Yt-mp4",
        _ => unreachable!(),
    }
}
