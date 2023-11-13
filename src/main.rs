mod helpers;
mod providers;

use helpers::{is_terminal::is_terminal, selection::selection};
use providers::allanime::allanime;
use std::{env, io::stdin, process::exit};

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
    let mut is_not_rofi = true;
    let mut sort_by_top = false;

    if is_terminal() {
        for arg in env::args().skip(1) {
            match arg.as_str() {
                "-h" | "--help" => {
                    version();

                    println!(
                        "\nUsage: eren <argument> <search query>

\t-h,  --help\t\t Print Help
\t-v,  --version\t\t Print version
\t-b,  --debug\t\t get vid link & other data
\t-s,  --sub\t\t dubbed video
\t-r,  --rofi\t\t use rofi
\t-g,  --get\t\t get vid link
\t-d,  --download\t\t download video with aria2
\t-t,  --top\t\t sort by top (get best search matches only)
\t-p=, --provider=\t change provider (Ak, Default, S-mp4, Sak, Luf-mp4) or (1, 2, 3, 4, 5)
\t-q=, --quality=\t\t change quality (2160, 1080, 720, 480, 360)"
                    );
                    exit(0);
                }
                "-v" | "--version" => {
                    version();
                    exit(0);
                }
                "-b" | "--debug" => todo = Todo::Debug,
                "-g" | "--get" => todo = Todo::GetLink,
                "-d" | "--download" => todo = Todo::Download,
                "-s" | "--sub" => sub = true,
                "-r" | "--rofi" => is_not_rofi = false,
                "-t" | "--top" => sort_by_top = true,
                arg if arg.starts_with("-q=") || arg.starts_with("--quality=") => {
                    quality = arg
                        .split_once('=')
                        .unwrap()
                        .1
                        .trim_end_matches('p')
                        .parse()
                        .expect("Quality must be a number");
                }
                arg if arg.starts_with("-p=") || arg.starts_with("--provider=") => {
                    let pro_str = arg.split_once('=').unwrap().1;

                    if let Ok(pro_num) = pro_str.parse() {
                        provider = pro_num
                    } else {
                        provider = provider_num(pro_str);
                    }
                }
                _ => {
                    query.push_str(&arg);
                    query.push(' ');
                }
            }
        }

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
        &query.into_boxed_str(), // String moved
        todo,
        provider,
        quality,
        sub,
        is_not_rofi,
        sort_by_top,
    )
    .await;
}

fn version() {
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
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
