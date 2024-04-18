mod helpers;
mod providers;

use clap::{arg, command, value_parser, ArgAction::SetTrue};
use helpers::{is_terminal::is_terminal, provider_num::provider_num, selection::selection};
use providers::allanime::allanime;
use std::{
    env::consts::OS,
    io::{stdin, stdout, Write},
    process::exit,
};

#[derive(Default, Debug, Clone)]
pub struct Vid {
    title: String,
    vid_link: String,
    audio_link: Option<String>,
    subtitle_path: Option<String>,
    referrer: Option<&'static str>,
    //user_agent: &'static str,
}

/*
impl Default for Vid {
    fn default() -> Self {
        Self {
            title: String::new(),
            vid_link: String::new(),
            audio_link: None,
            subtitle_link: String::new(),
            user_agent: "uwu",
            referrer: "",
        }
    }
}
*/

const YELLOW: &str = "\u{1b}[33m";
const RED: &str = "\u{1b}[31m";
const RESET: &str = "\u{1b}[0m";

#[derive(Clone, Copy)]
pub enum Todo {
    Play,
    Download,
    GetLink,
    Debug,
}

#[tokio::main]
async fn main() {
    let mut query = String::new();
    let mut todo = Todo::Play;
    let mut sub = false;
    let mut quality = 0;
    let mut provider = 1;
    let mut is_rofi = false;
    let mut sort_by_top = false;
    let matches = command!()
        .arg(arg!(-s --sub "Sets mode to sub").action(SetTrue))
        .arg(arg!(-r --rofi "Sets selection menu to rofi").action(SetTrue))
        .arg(arg!(-t --top "Sort by Top (gets best search results only)").action(SetTrue))
        .arg(
            arg!(-d --download "Downloads video using aria2")
                .conflicts_with_all(&["get", "debug"])
                .action(SetTrue),
        )
        .arg(
            arg!(-g --get "Gets video link")
                .conflicts_with_all(&["debug", "download"])
                .action(SetTrue),
        )
        .arg(
            arg!(-b --debug "Prints video link, audio link, etc")
                .conflicts_with_all(&["get", "download"])
                .action(SetTrue),
        )
        .arg(
            arg!(
                -q --quality <Resolution> "Sets desired resolution"
            )
            .required(false)
            .value_parser([
                "2160p", "1080p", "720p", "480p", "360p", "2160", "1080", "720", "480", "360",
            ]),
        )
        .arg(
            arg!(-p --provider <Provider> "Changes Provider")
                .required(false)
                .value_parser([
                    "Ak", "Default", "Sak", "S-mp4", "Luf-mp4", "Yt-mp4", "1", "2", "3", "4", "5",
                    "6",
                ]),
        )
        .arg(
            arg!([query] "Anime Name")
                .multiple_values(true)
                .value_parser(value_parser!(String)),
        )
        .get_matches();

    if let Some(pro) = matches.get_one::<String>("provider") {
        if let Ok(pro_num) = pro.parse() {
            provider = pro_num;
        } else {
            provider = provider_num(pro);
        }
    }

    if let Some(res) = matches.get_one::<String>("quality") {
        quality = res
            .trim_end_matches('p')
            .parse()
            .expect("Quality must be a number");
    }

    if matches.get_flag("sub") {
        sub = true;
    }

    if matches.get_flag("download") {
        todo = Todo::Download;
    } else if matches.get_flag("debug") {
        todo = Todo::Debug;
    } else if matches.get_flag("get") {
        todo = Todo::GetLink;
    // provider 1 has separate audio & sub link & 2 has referer which cannot be passed from termux
    } else if OS == "android" && matches!(provider, 1 | 6) {
        provider = 2;
    }

    if matches.get_flag("rofi") {
        is_rofi = true;
    }

    if matches.get_flag("top") {
        sort_by_top = true;
    }

    if let Some(anime) = matches.get_many("query") {
        query = anime.cloned().collect::<Vec<String>>().join(" ");
    }

    drop(matches);

    if !is_terminal() {
        is_rofi = true;
    }

    if query.trim().is_empty() {
        if !is_rofi {
            print!("{YELLOW}Search a Cartoon/Anime: {RESET}");
            stdout().flush().expect("Failed to flush stdout");
            stdin().read_line(&mut query).expect("Failed to read line");

            query = query
                .trim_end_matches(|ch| ch == '\n' || ch == ' ')
                .to_owned();
        } else {
            query = selection("", "Search a Cartoon/Anime: ", false, is_rofi);
        }

        if query.trim().is_empty() {
            exit(0);
        }
    }

    let query = query.into_boxed_str();

    if let Err(err) = allanime(&query, todo, provider, quality, sub, is_rofi, sort_by_top).await {
        println!("{RED}Error:{RESET} {err}");
    }
}
