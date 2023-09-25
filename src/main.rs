mod helpers;
mod providers;

use std::{env, io::stdin, process::exit};

use helpers::{is_terminal::is_terminal, selection::selection};
use providers::allanime::allanime;

#[derive(Debug)]
pub struct Vid {
    title: String,
    user_agent: String,
    vid_link: String,
    audio_link: String,
    subtitle_link: String,
    referrer: String,
}

impl Default for Vid {
    fn default() -> Self {
        Self {
            title: String::new(),
            user_agent: String::from("uwu"),
            vid_link: String::new(),
            audio_link: String::new(),
            subtitle_link: String::new(),
            referrer: String::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    let mut query = String::new();
    let mut todo = String::from("play");
    let mut sub = false;
    let mut quality = String::new();
    let mut provider = String::from("Ak");
    let mut is_not_rofi = true;
    let mut sort_by_top = false;

    if is_terminal() {
        env::args().skip(1).for_each(|arg| match arg.as_str() {
            "-h" | "--help" => {
                println!(
                    "\nUsage :\teren <argument> <search query>\n
\t-h | --help\t\t Print Help
\t-b | --debug\t\t get vid link & other data
\t-s | --sub\t\t dubbed video
\t-r | --rofi\t\t use rofi                    
\t-g | --get\t\t get vid link
\t-d | --download\t download                    
\t-t | --top\t\t sort by top (get best search matches only)
\t-p= | --provider=\t change provider (Ak, Default, S-mp4, Sak, Luf-mp4)
\t-q= | --quality=\t change quality(2160, 1080, 720, 480, 360)"
                );
                exit(0);
            }
            "-b" | "--debug" => todo = String::from("debug"),
            "-g" | "--get" => todo = String::from("print link"),
            "-d" | "--download" => todo = String::from("download"),
            "-s" | "--sub" => sub = true,
            "-r" | "--rofi" => is_not_rofi = false,
            "-t" | "--top" => sort_by_top = true,
            a if a.starts_with("-q=") || a.starts_with("--quality=") => {
                quality = arg.split_once('=').unwrap().1.to_string()
            }
            a if a.starts_with("-p=") || a.starts_with("--provider=") => {
                provider = arg.split_once('=').unwrap().1.to_string()
            }
            _ => {
                query.push_str(&arg);
                query.push(' ');
            }
        });

        while query.trim().is_empty() {
            println!("Search a Cartoon/Anime");
            stdin().read_line(&mut query).expect("Failed to read line");
        }
        query = query
            .trim_end_matches(|ch| ch == '\n' || ch == ' ')
            .to_string();
    } else {
        is_not_rofi = false;

        query = selection("", "Search Anime: ", false, is_not_rofi);
        if query.is_empty() {
            exit(0);
        }
    }

    allanime(
        &query,
        todo,
        &provider,
        &quality,
        sub,
        is_not_rofi,
        sort_by_top,
    )
    .await;
}
