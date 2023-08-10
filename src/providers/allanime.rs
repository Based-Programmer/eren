use crate::{
    helpers::{play_manager::play_manage, reqwests::*, selection::selection},
    Vid,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use std::{process::exit, sync::Arc};
use tokio::{
    sync::{mpsc, Mutex},
    task,
};
use url::form_urlencoded::byte_serialize;

const USER_AGENT: &str = "uwu";
const REFERER: &str = "https://allanime.to";
const RED: &str = "\u{1b}[31m";
const RESET: &str = "\u{1b}[0m";
//const GREEN: &str = "\u{1b}[32m";
//const YELLOW: &str = "\u{1b}[33m";

pub async fn allanime(
    query: &str,
    todo: String,
    mode: &str,
    provider: &str,
    quality: &str,
    is_not_rofi: bool,
    sort_by_top: bool,
) {
    let resp = search(query, mode, sort_by_top).await;

    let mut ids = Vec::new();
    let mut numbered_name = String::new();
    let mut episodes = Vec::new();

    let data: Value = serde_json::from_str(&resp).expect("Failed to derive json");

    if let Some(shows) = data["data"]["shows"]["edges"].as_array() {
        if shows.is_empty() {
            eprintln!("{}No result{}", RED, RESET);
            exit(1);
        }

        let mut count = 1;
        let mut newline = "";

        shows.iter().for_each(|show| {
            ids.push(show["_id"].as_str().expect("id wasn't found"));

            if let Some(name) = show["englishName"].as_str() {
                numbered_name = format!("{numbered_name}{newline}{count} {name}");
            } else {
                let name = show["name"].as_str().expect("name wasn't found");
                numbered_name = format!("{numbered_name}{newline}{count} {name}");
            }
            count += 1;
            newline = "\n";

            if let Some(ep) = show["availableEpisodesDetail"][mode].as_array() {
                let episode = ep
                    .iter()
                    .map(|episode| episode.as_str().unwrap().trim_matches('"'))
                    .rev()
                    .collect::<Vec<_>>();
                episodes.push(episode);
            }
        });
    }
    let selected = selection(&numbered_name, "Select anime: ", false, is_not_rofi);
    let (index, anime_name) = selected.split_once(' ').expect("Failed to split");

    let index = index.parse::<u8>().expect("Failed to parse index") - 1;
    let id = ids[index as usize];
    let episode_vec = &episodes[index as usize];
    let episode = &episode_vec.join("\n");
    let total_episodes = episode_vec.len();

    let mut choice = String::new();
    let mut episode_index = 0;

    while choice != *"quit" {
        let start = match choice.as_str() {
            "next" => episode_vec[episode_index].to_string(),
            "previous" => episode_vec[episode_index - 2].to_string(),
            "replay" => episode_vec[episode_index - 1].to_string(),
            _ => selection(episode, "Select episode: ", true, is_not_rofi),
        };

        let mut start = start.lines();
        let end = start.clone().last().unwrap();
        let start = start.next().unwrap();

        episode_index = episode.lines().position(|x| x == start).unwrap();

        let (sender, mut receiver) = mpsc::channel(1);

        let todo = todo.clone();

        let play_task = task::spawn(async move {
            while let Some(video) = receiver.recv().await {
                let todo_mutex = Arc::new(Mutex::new(&todo));
                play_manage(video, todo_mutex).await;
            }
        });

        let mut current_ep = "";

        while current_ep != end {
            current_ep = episode.lines().nth(episode_index).unwrap();

            let mut vid = Vid {
                title: format!("{} Episode {}", anime_name, current_ep),
                ..Default::default()
            };

            let resp = get_episodes(id, current_ep, mode).await;

            let source: Value = serde_json::from_str(&resp).expect("Failed to derive json");

            let mut source_url = Vec::new();
            let mut source_name = Vec::new();

            if let Some(source_urls) = source["data"]["episode"]["sourceUrls"].as_array() {
                source_urls.iter().for_each(|url| {
                    let name = url["sourceName"].as_str().expect("sourceName wasn't found");
                    let link = url["sourceUrl"]
                        .as_str()
                        .expect("sourceUrl wasn't found")
                        .to_string();

                    if name == "Default"
                        || name == "S-mp4"
                        || name == "Sak"
                        || name == "Luf-mp4"
                        || name == "Ak"
                    {
                        let decoded_link = decrypt_allanime(&link);

                        source_url.push(decoded_link);
                        source_name.push(name);
                    }
                });
            }

            vid = get_link(source_name, source_url, provider, quality, vid).await;
            sender.send(vid).await.expect("Failed to send link");

            episode_index += 1;
        }

        drop(sender);

        play_task.await.expect("Play task panicked");

        if episode_index == 1 && episode_index == total_episodes {
            choice = selection("quit\nreplay", "Enter a choice: ", false, is_not_rofi);
        } else if episode_index == 1 {
            choice = selection(
                "quit\nnext\nselect\nreplay",
                "Enter a choice: ",
                false,
                is_not_rofi,
            );
        } else if episode_index == total_episodes {
            choice = selection(
                "quit\nprevious\nselect\nreplay",
                "Enter a choice: ",
                false,
                is_not_rofi,
            );
        } else {
            choice = selection(
                "quit\nnext\nprevious\nselect\nreplay",
                "Enter a choice: ",
                false,
                is_not_rofi,
            );
        }
    }
}

async fn search(query: &str, mode: &str, sort_by_top: bool) -> String {
    const SEARCH_GQL: &str = "query (
        $search: SearchInput
        $translationType: VaildTranslationTypeEnumType
        $countryOrigin: VaildCountryOriginEnumType
    ) {
        shows(
            search: $search
            limit: 40
            page: 1
            translationType: $translationType
            countryOrigin: $countryOrigin
        ) {
            edges {
                _id
        name
                englishName
        availableEpisodes
        availableEpisodesDetail
            }
        }
    }";

    let sort = if sort_by_top {
        r#""sortBy":"Top","#
    } else {
        ""
    };

    let variables = format!(
        r#"{{"search":{{{}"allowAdult":true,"allowUnknown":true,"query":"{}"}},"translationType":"{}"}}"#,
        sort, query, mode,
    );

    let link = format!(
        "https://api.allanime.day/allanimeapi?variables={}&query={}",
        byte_serialize(variables.as_bytes()).collect::<String>(),
        byte_serialize(SEARCH_GQL.as_bytes()).collect::<String>()
    );

    get_isahc(&link, USER_AGENT, REFERER).await
}

async fn get_episodes(id: &str, episode_num: &str, mode: &str) -> String {
    const EPISODES_GQL: &str = "query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {
    episode(
        showId: $showId
        translationType: $translationType
        episodeString: $episodeString
    ) {
        sourceUrls
    }
}";

    let variables = format!(
        r#"{{"showId":"{}","translationType":"{}","episodeString":"{}"}}"#,
        id, mode, episode_num
    );

    let link = format!(
        "https://api.allanime.day/allanimeapi?variables={}&query={}",
        byte_serialize(variables.as_bytes()).collect::<String>(),
        byte_serialize(EPISODES_GQL.as_bytes()).collect::<String>()
    );

    get_isahc(
        &link,
        "Mozilla/5.0 (X11; Linux x86_64; rv:99.0) Gecko/20100101 Firefox/100.0",
        REFERER,
    )
    .await
}

fn decrypt_allanime(link: &str) -> String {
    let hex = link.trim_start_matches("##");
    const PASSWORD: &str = "1234567890123456789";
    let data = hex::decode(hex).unwrap();

    let genexp = || {
        data.iter().map(|segment| {
            let mut segment = *segment;

            PASSWORD.chars().for_each(|char| {
                segment ^= char as u8;
            });
            segment as char
        })
    };

    genexp().collect::<String>().replace(
        "/apivtwo/clock?id=",
        "https://embed.ssbcontent.site/apivtwo/clock.json?id=",
    )
}

async fn get_link(
    source_name: Vec<&str>,
    source_url: Vec<String>,
    provider: &str,
    quality: &str,
    mut vid: Vid,
) -> Vid {
    if provider == "Ak" && source_name.contains(&"Ak") {
        let v = get_json("Ak", source_name, source_url).await;

        if let Some(vid_link) = v["links"][0]["rawUrls"]["vids"].as_array() {
            let mut vid_check = "";

            if !quality.is_empty() {
                vid_link.iter().position(|video| {
                    if quality.parse::<u16>().unwrap() == video["height"] {
                        vid_check = video["url"].as_str().expect("Failed to get Ak video link");
                        true
                    } else {
                        false
                    }
                });
            }

            if vid_check.is_empty() {
                let video = &vid_link[0];
                vid_check = video["url"]
                    .as_str()
                    .expect("Failed to get Ak best video link");
            }
            vid.vid_link = vid_check.trim_matches('"').to_string();

            vid.audio_link = v["links"][0]["rawUrls"]["audios"][0]["url"]
                .as_str()
                .expect("Failed to get Ak audio link")
                .trim_matches('"')
                .to_string();

            vid.subtitle_link = v["links"][0]["subtitles"][0]["src"]
                .as_str()
                .expect("Failed to get Ak subtitle link")
                .trim_matches('"')
                .to_string();
        }
    } else if { provider == "Default" || provider == "Ak" } && source_name.contains(&"Default") {
        let pro = "Default";
        let v = get_json(pro, source_name, source_url).await;
        let link = default_link(v, pro);

        if !quality.is_empty() {
            let re = Regex::new(&format!(
                r#"https://repackager.wixmp.com/(.*/)[^/]*{}p[^/]*(/mp4/file\.mp4)"#,
                quality
            ))
            .unwrap();

            if let Some(captures) = re.captures(&link) {
                vid.vid_link = format!("https://{}{}p{}", &captures[1], quality, &captures[2]);
            }
        }
        if vid.vid_link.is_empty() {
            static RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r#"https://repackager.wixmp.com/(.*/),([^,]*)[^/]*(/mp4/file\.mp4)"#)
                    .unwrap()
            });
            vid.vid_link = format!(
                "https://{}{}{}",
                &RE.captures(&link).unwrap()[1],
                &RE.captures(&link).unwrap()[2],
                &RE.captures(&link).unwrap()[3],
            );
        }
    } else if { provider != "Sak" && provider != "Luf-mp4" } && source_name.contains(&"S-mp4") {
        let pro = "S-mp4";
        let v = get_json(pro, source_name, source_url).await;
        vid.vid_link = default_link(v, pro);
    } else if provider != "Luf-mp4" && source_name.contains(&"Sak") {
        let pro = "Sak";
        let v = get_json(pro, source_name, source_url).await;
        vid.vid_link = default_link(v, pro);
    } else {
        let pro = "Luf-mp4";
        let v = get_json(pro, source_name, source_url).await;
        let link = default_link(v, pro);

        let resp = get_isahc(&link, USER_AGENT, &link).await;
        let mut line = String::new();

        if !quality.is_empty() {
            let re = Regex::new(&format!(r#"(ep\..*\.{}\.m3u8)"#, quality)).unwrap();
            if let Some(captures) = re.captures(&resp) {
                line = captures[1].to_string();
            }
        }
        if vid.vid_link.is_empty() && line.is_empty() {
            line = resp.lines().last().unwrap().to_string();
        }
        let split_link = link.rsplit_once('/').unwrap().0;
        vid.vid_link = format!("{}/{}", split_link, line);
    }
    vid
}

async fn get_json(provider: &str, source_name: Vec<&str>, source_url: Vec<String>) -> Value {
    let index = source_name
        .iter()
        .position(|item| item == &provider)
        .unwrap();

    let link = &source_url[index];

    let resp = get_isahc(link, USER_AGENT, REFERER).await;

    serde_json::from_str(&resp).unwrap()
}

fn default_link(v: Value, provider: &str) -> String {
    if let Some(link) = v["links"][0]["link"].as_str() {
        link.to_string()
    } else {
        eprintln!("{}No link from {} provider{}", RED, provider, RESET);
        exit(1);
    }
}
