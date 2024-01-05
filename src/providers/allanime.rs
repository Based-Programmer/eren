use crate::{
    helpers::{
        play_manager::play_manage,
        reqwests::{client, get_isahc},
        selection::selection,
    },
    Todo, Vid,
};

use isahc::HttpClient;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use std::{error::Error, process::exit};
use tokio::{sync::mpsc, task};
use url::form_urlencoded::byte_serialize;

const ALLANIME_API: &str = "https://api.allanime.day/api";
const RED: &str = "\u{1b}[31m";
const RESET: &str = "\u{1b}[0m";

pub async fn allanime(
    query: &str,
    todo: Todo,
    provider: u8,
    quality: u16,
    sub: bool,
    is_rofi: bool,
    sort_by_top: bool,
) -> Result<(), Box<dyn Error>> {
    let mode = if sub { "sub" } else { "dub" };
    let client = &client("uwu", "https://allanime.to")?;

    let search_data: Value = {
        let resp = search(query, mode, sort_by_top, client).await?;
        serde_json::from_str(&resp).expect("Failed to derive json from search resp")
    };

    let mut ids: Vec<Box<str>> = Vec::new();
    let mut episodes = Vec::new();

    let anime_names = {
        let mut anime_names = Vec::new();

        if let Some(shows) = search_data["data"]["shows"]["edges"].as_array() {
            if shows.is_empty() {
                eprintln!("{}No result{}", RED, RESET);
                exit(1);
            }

            for (i, show) in shows.iter().enumerate() {
                ids.push(show["_id"].as_str().expect("id wasn't found").into());

                if let Some(name) = show["englishName"].as_str() {
                    anime_names.push(format!("{i} {name}"));
                } else {
                    let name = show["name"].as_str().expect("name wasn't found");
                    anime_names.push(format!("{i} {name}"));
                }

                if let Some(ep) = show["availableEpisodesDetail"][mode].as_array() {
                    let episode: Box<[Box<str>]> = ep
                        .iter()
                        .map(|episode| episode.as_str().unwrap().trim_matches('"').into())
                        .rev()
                        .collect();
                    episodes.push(episode);
                }
            }
        }
        anime_names.join("\n").into_boxed_str()
    };

    drop(search_data);
    let episodes = episodes.into_boxed_slice();
    let ids = ids.into_boxed_slice();

    let selected = selection(&anime_names, "Select anime: ", false, is_rofi);
    drop(anime_names);

    let (index, anime_name) = selected
        .split_once(' ')
        .expect("Failed to split index & anime name");

    let index = index.parse::<u8>().expect("Failed to parse index");
    let id = ids[index as usize].clone();
    let episode = episodes[index as usize].join("\n").into_boxed_str();
    let episode_vec: Box<[&str]> = episode.lines().collect();
    let total_episodes = episode_vec.len() as u16;
    let mut choice = String::new();
    let mut episode_index: u16 = 0;

    drop(ids);
    drop(episodes);

    while choice != "quit" {
        let start_string = match choice.as_str() {
            "next" => episode_vec[episode_index as usize].to_string(),
            "previous" => episode_vec[episode_index as usize - 2].to_string(),
            "replay" => episode_vec[episode_index as usize - 1].to_string(),
            _ => selection(&episode, "Select episode: ", true, is_rofi),
        };

        let start: Vec<&str> = start_string.lines().collect();
        let end = start.last().unwrap().to_string();

        episode_index = episode_vec.iter().position(|x| **x == *start[0]).unwrap() as u16;
        drop(start);
        drop(start_string);

        let (sender, mut receiver) = mpsc::channel(1);

        let play_task = task::spawn(async move {
            while let Some(video) = receiver.recv().await {
                play_manage(video, todo).await;
            }
        });

        let mut current_ep = "";

        while current_ep != end {
            current_ep = &episode_vec[episode_index as usize];

            let mut vid = Vid {
                title: format!("{} Episode {}", anime_name, current_ep),
                ..Default::default()
            };

            let source: Value = {
                let resp = get_episodes(client, &id, current_ep, mode).await?;
                serde_json::from_str(&resp).expect("Failed to derive json from episode response")
            };

            let mut source_url = Vec::new();
            let mut source_name = Vec::new();

            if let Some(sources) = source["data"]["episode"]["sourceUrls"].as_array() {
                for source in sources {
                    let name = source["sourceName"]
                        .as_str()
                        .expect("sourceName wasn't found")
                        .to_string();

                    let url = source["sourceUrl"]
                        .as_str()
                        .expect("sourceUrl wasn't found");

                    if name == "Default"
                        || name == "S-mp4"
                        || name == "Sak"
                        || name == "Luf-mp4"
                        || name == "Ak"
                    {
                        let decoded_link = decrypt_allanime(url);

                        source_url.push(decoded_link);
                        source_name.push(name);
                    }
                }
            }
            drop(source);

            vid =
                get_streaming_link(client, source_name, source_url, provider, quality, vid).await?;
            sender.send(vid).await.expect("Failed to send link");

            episode_index += 1;
        }

        drop(sender);
        play_task.await.expect("Play task failed");

        if episode_index == 1 && episode_index == total_episodes {
            choice = choice_selection("quit\nreplay", is_rofi);
        } else if episode_index == 1 {
            choice = choice_selection("quit\nnext\nselect\nreplay", is_rofi);
        } else if episode_index == total_episodes {
            choice = choice_selection("quit\nprevious\nselect\nreplay", is_rofi);
        } else {
            choice = choice_selection("quit\nnext\nprevious\nselect\nreplay", is_rofi);
        }
    }

    Ok(())
}

async fn search(
    query: &str,
    mode: &str,
    sort_by_top: bool,
    client: &HttpClient,
) -> Result<Box<str>, Box<dyn Error>> {
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
        "{}?variables={}&query={}",
        ALLANIME_API,
        byte_serialize(variables.as_bytes()).collect::<String>(),
        byte_serialize(SEARCH_GQL.as_bytes()).collect::<String>()
    );

    get_isahc(client, &link).await
}

async fn get_episodes(
    client: &HttpClient,
    id: &str,
    episode_num: &str,
    mode: &str,
) -> Result<Box<str>, Box<dyn Error>> {
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
        "{}?variables={}&query={}",
        ALLANIME_API,
        byte_serialize(variables.as_bytes()).collect::<String>(),
        byte_serialize(EPISODES_GQL.as_bytes()).collect::<String>()
    );

    get_isahc(client, &link).await
}

fn decrypt_allanime(source_url: &str) -> String {
    let decoded_link: String = hex::decode(&source_url[2..])
        .expect("Failed to decode hex")
        .into_iter()
        .map(|segment| (segment ^ 56) as char)
        .collect();

    decoded_link.replace(
        "/apivtwo/clock?id=",
        "https://allanime.day/apivtwo/clock.json?id=",
    )
}

async fn get_streaming_link(
    client: &HttpClient,
    source_name: Vec<String>,
    source_url: Vec<String>,
    provider: u8,
    mut quality: u16,
    mut vid: Vid,
) -> Result<Vid, Box<dyn Error>> {
    if provider == 1 && source_name.contains(&String::from("Ak")) {
        let v = get_json(client, "Ak", source_name, source_url).await?;

        if let Some(vid_link) = v["links"][0]["rawUrls"]["vids"].as_array() {
            let mut vid_check = "";

            if quality != 0 {
                for video in vid_link {
                    if quality == video["height"] {
                        vid_check = video["url"].as_str().unwrap_or_else(|| {
                            eprintln!(
                                "{}Failed to get vid from provider Ak by quality{}",
                                RED, RESET
                            );
                            ""
                        });
                        break;
                    }
                }
            }

            if vid_check.is_empty() {
                let video = &vid_link[0];
                vid_check = video["url"]
                    .as_str()
                    .expect("Failed to get Ak best video link");
            }
            vid.vid_link = vid_check.trim_matches('"').to_string();

            vid.audio_link = Some(
                v["links"][0]["rawUrls"]["audios"][0]["url"]
                    .as_str()
                    .expect("Failed to get Ak audio link")
                    .trim_matches('"')
                    .to_string(),
            );

            vid.subtitle_link = Some(
                v["links"][0]["subtitles"][0]["src"]
                    .as_str()
                    .expect("Failed to get Ak subtitle link")
                    .trim_matches('"')
                    .to_string(),
            );
        }
    } else if { provider <= 2 } && source_name.contains(&String::from("Default")) {
        let pro = "Default";
        let v = get_json(client, pro, source_name, source_url).await?;
        let link = default_link(v, pro);

        if quality == 0 || quality > 1080 {
            quality = 1080
        }

        let re = Regex::new(&format!(
            r"https://repackager.wixmp.com/(.*/)[^/]*{}p[^/]*(/mp4/file\.mp4)",
            quality
        ))
        .unwrap();

        if let Some(captures) = re.captures(&link) {
            vid.vid_link = format!("https://{}{}p{}", &captures[1], quality, &captures[2]);
        }

        if vid.vid_link.is_empty() {
            static RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"https://repackager.wixmp.com/(.*/)[^/]*,([0-9]*p),(/mp4/file\.mp4)")
                    .unwrap()
            });

            let cap = RE
                .captures(&link)
                .expect("Failed to get video link from wixmp");
            vid.vid_link = format!("https://{}{}{}", &cap[1], &cap[2], &cap[3],);
        }
    } else if { provider <= 3 } && source_name.contains(&String::from("S-mp4")) {
        let pro = "S-mp4";
        let v = get_json(client, pro, source_name, source_url).await?;
        vid.vid_link = default_link(v, pro);
    } else if provider <= 4 && source_name.contains(&String::from("Sak")) {
        let pro = "Sak";
        let v = get_json(client, pro, source_name, source_url).await?;
        vid.vid_link = default_link(v, pro);
    } else {
        let pro = "Luf-mp4";
        let v = get_json(client, pro, source_name, source_url).await?;
        let link = default_link(v, pro).into_boxed_str();

        let resp = get_isahc(client, &link).await?;
        let mut m3u8 = String::new();

        if quality != 0 {
            let re = Regex::new(&format!(r"(ep\..*\.{}\.m3u8)", quality)).unwrap();
            if let Some(captures) = re.captures(&resp) {
                m3u8 = captures[1].to_string();
            }
        }
        if vid.vid_link.is_empty() && m3u8.is_empty() {
            m3u8 = resp.lines().last().unwrap().to_string();
        }
        let split_link = link.rsplit_once('/').unwrap().0;
        vid.vid_link = format!("{}/{}", split_link, m3u8);
    }

    Ok(vid)
}

async fn get_json(
    client: &HttpClient,
    provider: &str,
    source_name: Vec<String>,
    source_url: Vec<String>,
) -> Result<Value, Box<dyn Error>> {
    let index = source_name
        .iter()
        .position(|item| item == provider)
        .unwrap();
    let url = source_url[index].clone();

    drop(source_name);
    drop(source_url);

    let resp = get_isahc(client, &url).await?;
    Ok(serde_json::from_str(&resp).expect("Failed to derive json"))
}

fn default_link(v: Value, provider: &str) -> String {
    match v["links"][0]["link"].as_str() {
        Some(link) => link.to_string(),
        None => {
            eprintln!("{}No link from {} provider{}", RED, provider, RESET);
            exit(1);
        }
    }
}

fn choice_selection(select: &str, is_rofi: bool) -> String {
    selection(select, "Enter a choice: ", false, is_rofi)
}
