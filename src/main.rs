mod helpers;
mod providers;

use helpers::{is_terminal::is_terminal, selection::selection};
use providers::allanime::allanime;
use std::{
    env,
    io::{stdin, stdout, Write},
    process::exit,
};

use clap::{arg, command, value_parser, ArgAction::SetTrue};

#[derive(Default, Debug, Clone)]
pub struct Vid {
    title: String,
    vid_link: String,
    audio_link: Option<String>,
    subtitle_link: Option<String>,
    //user_agent: &'static str,
    //referrer: &'static str,
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
            .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(-p --provider <Provider> "Changes Provider")
                .required(false)
                .value_parser([
                    "Ak", "Default", "S-mp4", "Sak", "Luf-mp4", "1", "2", "3", "4", "5",
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

    if matches.get_flag("debug") {
        todo = Todo::Debug;
    }

    if matches.get_flag("download") {
        todo = Todo::Download;
    }

    if matches.get_flag("get") {
        todo = Todo::GetLink;
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

fn provider_num(provider: &str) -> u8 {
    match provider {
        "Default" => 2,
        "S-mp4" => 3,
        "Sak" => 4,
        "Luf-mp4" => 5,
        _ => 1,
    }
}
