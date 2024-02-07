use crate::{
    helpers::{play_manager::play_manage, reqwests::*, selection::selection},
    Todo, Vid,
};

use isahc::HttpClient;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use std::{error::Error, io::ErrorKind, process::exit};
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

    let search_data = search(query, mode, sort_by_top, client)?;

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

                let available_ep = show["availableEpisodes"][mode]
                    .as_u64()
                    .expect("'Available Episodes' wasn't found");

                if let Some(name) = show["englishName"].as_str() {
                    anime_names.push(format!("{i} {name} ({available_ep} Episodes)"));
                } else {
                    let name = show["name"].as_str().expect("name wasn't found");
                    anime_names.push(format!("{i} {name} ({available_ep} Episodes)"));
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

    let anime_name = anime_name.rsplit_once(" (").unwrap().0.to_owned();
    let index = index.parse::<u8>()?;
    let id = ids[index as usize].clone();
    let episode = episodes[index as usize].join("\n").into_boxed_str();
    let episode_vec: Box<[&str]> = episode.lines().collect();
    let total_episodes = episode_vec.len() as u16;
    let mut choice = String::new();
    let mut episode_index: u16 = 0;

    drop(selected);
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

        episode_index = episode_vec.iter().position(|&x| x == start[0]).unwrap() as u16;
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
            current_ep = episode_vec[episode_index as usize];

            let source = get_episodes(client, &id, current_ep, mode)?;

            let mut vid = Vid::default();

            if let Some(mut ep_title) = source["data"]["episode"]["episodeInfo"]["notes"].as_str() {
                ep_title = ep_title
                    .split_once("<note-split>")
                    .unwrap_or((ep_title, ""))
                    .0;

                vid.title = if ep_title != "Untitled" {
                    format!("{} Episode {} - {}", anime_name, current_ep, ep_title)
                } else {
                    format!("{} Episode {}", anime_name, current_ep)
                };
            } else if total_episodes > 1 {
                vid.title = format!("{} Episode {}", anime_name, current_ep);
            } else {
                vid.title = anime_name.to_owned();
            }

            let mut source_url = Vec::new();
            let mut source_name = Vec::new();

            if let Some(sources) = source["data"]["episode"]["sourceUrls"].as_array() {
                for source in sources {
                    let name = source["sourceName"]
                        .as_str()
                        .expect("sourceName wasn't found");

                    let url = source["sourceUrl"]
                        .as_str()
                        .expect("sourceUrl wasn't found");

                    if name == "Default"
                        || name == "S-mp4"
                        || name == "Sak"
                        || name == "Luf-mp4"
                        || name == "Ak"
                    {
                        match decrypt_allanime(url) {
                            Ok(decoded_link) => source_url.push(decoded_link),
                            Err(err) => eprintln!("{RED}Error:{RESET} {err}"),
                        }

                        let name_num: u8 = match name {
                            "Ak" => 1,
                            "Default" => 2,
                            "S-mp4" => 3,
                            "Sak" => 4,
                            "Luf-mp4" => 5,
                            _ => 0,
                        };

                        if name_num != 0 {
                            source_name.push(name_num);
                        }
                    }
                }
            }
            drop(source);

            vid = get_streaming_link(client, &source_name, &source_url, provider, quality, vid)?;

            drop(source_name);
            drop(source_url);

            sender.send(vid).await?;

            episode_index += 1;
        }

        drop(sender);
        play_task.await?;

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

fn search(
    query: &str,
    mode: &str,
    sort_by_top: bool,
    client: &HttpClient,
) -> Result<Value, Box<dyn Error>> {
    const SEARCH_GQL: &str = "query (
        $search: SearchInput
        $translationType: VaildTranslationTypeEnumType
    ) {
        shows(
            search: $search
            limit: 40
            page: 1
            translationType: $translationType
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

    get_isahc_json(client, &link)
}

fn get_episodes(
    client: &HttpClient,
    id: &str,
    episode_num: &str,
    mode: &str,
) -> Result<Value, Box<dyn Error>> {
    const EPISODES_GQL: &str = "query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {
    episode(
        showId: $showId
        translationType: $translationType
        episodeString: $episodeString
    ) {
        episodeString
        sourceUrls
        episodeInfo {
            notes
        }
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

    get_isahc_json(client, &link)
}

fn decrypt_allanime(source_url: &str) -> Result<String, Box<dyn Error>> {
    let decoded_link: String = hex::decode(&source_url[2..])?
        .into_iter()
        .map(|segment| (segment ^ 56) as char)
        .collect();

    Ok(decoded_link.replace(
        "/apivtwo/clock?id=",
        "https://allanime.day/apivtwo/clock.json?id=",
    ))
}

fn get_streaming_link(
    client: &HttpClient,
    source_name: &[u8],
    source_url: &[String],
    mut provider: u8,
    mut quality: u16,
    mut vid: Vid,
) -> Result<Vid, Box<dyn Error>> {
    let mut count: u8 = 0;

    while vid.vid_link.is_empty() && count < 5 {
        if source_name.contains(&provider) {
            match provider {
                1 => {
                    let v = get_json(client, provider, source_name, source_url)?;

                    if let Some(vid_link) = v["links"][0]["rawUrls"]["vids"].as_array() {
                        if quality != 0 {
                            for video in vid_link {
                                if quality == video["height"] {
                                    match video["url"].as_str() {
                                        Some(vid_url) => vid.vid_link = vid_url.to_owned(),
                                       None => eprintln!("{RED}Failed to desired quality from provider Ak{RESET}"),
                                    }
                                    break;
                                }
                            }
                        }

                        if vid.vid_link.is_empty() {
                            vid.vid_link = vid_link[0]["url"]
                                .as_str()
                                .expect("Failed to get Ak best video link")
                                .to_owned();
                        }
                        vid.vid_link = vid.vid_link.trim_matches('"').to_owned();

                        vid.audio_link = Some(
                            v["links"][0]["rawUrls"]["audios"][0]["url"]
                                .as_str()
                                .expect("Failed to get Ak audio link")
                                .trim_matches('"')
                                .to_owned(),
                        );

                        vid.subtitle_link = Some(
                            v["links"][0]["subtitles"][0]["src"]
                                .as_str()
                                .expect("Failed to get Ak subtitle link")
                                .trim_matches('"')
                                .to_owned(),
                        );
                    }
                }
                2 => {
                    let v = get_json(client, provider, source_name, source_url)?;
                    if let Some(link) = v["links"][0]["link"].as_str() {
                        if quality == 0 || quality > 1080 {
                            quality = 1080
                        }

                        let re = Regex::new(&format!(
                            r"https://repackager.wixmp.com/(.*/)[^/]*{}p[^/]*(/mp4/file\.mp4)",
                            quality
                        ))?;

                        if let Some(captures) = re.captures(link) {
                            vid.vid_link =
                                format!("https://{}{}p{}", &captures[1], quality, &captures[2]);
                        }

                        if vid.vid_link.is_empty() {
                            static RE: Lazy<Regex> = Lazy::new(|| {
                                Regex::new(
                            r"https://repackager.wixmp.com/(.*/)[^/]*,([0-9]*p),(/mp4/file\.mp4)",
                        )
                        .unwrap()
                            });

                            let cap = RE
                                .captures(link)
                                .expect("Failed to get video link from wixmp");
                            vid.vid_link = format!("https://{}{}{}", &cap[1], &cap[2], &cap[3],);
                        }
                    }
                }
                3 => {
                    let v = get_json(client, provider, source_name, source_url)?;
                    vid.vid_link = default_link(v);
                }
                4 => {
                    let v = get_json(client, provider, source_name, source_url)?;
                    vid.vid_link = default_link(v);
                }
                _ => {
                    let v = get_json(client, provider, source_name, source_url)?;
                    if let Some(link) = v["links"][0]["link"].as_str() {
                        let resp = get_isahc(client, link)?;
                        let mut m3u8 = String::new();

                        if quality != 0 {
                            let re = Regex::new(&format!(r"(ep\..*\.{}\.m3u8)", quality))?;
                            if let Some(captures) = re.captures(&resp) {
                                m3u8 = captures[1].to_owned();
                            }
                        }

                        if vid.vid_link.is_empty() && m3u8.is_empty() {
                            m3u8 = resp.lines().last().unwrap().to_owned();
                        }

                        let split_link = link.rsplit_once('/').unwrap().0;
                        vid.vid_link = format!("{}/{}", split_link, m3u8);
                    }
                }
            }
        }
        provider = provider % 5 + 1;
        count += 1;
    }

    if vid.vid_link.is_empty() {
        Err(Box::new(std::io::Error::new(
            ErrorKind::InvalidInput,
            "No video link was found",
        )))
    } else {
        Ok(vid)
    }
}

fn get_json(
    client: &HttpClient,
    provider: u8,
    source_name: &[u8],
    source_url: &[String],
) -> Result<Value, Box<dyn Error>> {
    let index = source_name
        .iter()
        .position(|&item| item == provider)
        .expect("No supported provider");

    let url = source_url[index].clone();

    get_isahc_json(client, &url)
}

fn default_link(v: Value) -> String {
    v["links"][0]["link"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn choice_selection(select: &str, is_rofi: bool) -> String {
    selection(select, "Enter a choice: ", false, is_rofi)
}
