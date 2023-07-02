mod helpers;
mod providers;

use std::{env, process::exit};

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
    let mut todo = "play";
    let mut mode = "sub";
    let mut quality = String::new();
    let mut provider = String::from("Ak");

    env::args().skip(1).for_each(|arg| match arg.as_str() {
        "-h" | "--help" => {
            println!(
                "\nUsage :\teren <argument> <search query>\n
\t-h | --help\t\t Print Help
\t-S | --sub\t\t subbed video
\t-D | --dub\t\t dubbed video
\t-D | --debug\t\t get vid link & other data
\t-g | --get\t\t get vid link
\t-p= | --provider=\t change provider (Ak, Default, S-mp4, Sak, Luf-mp4)
\t-q= | --quality=\t change quality(2160, 1080, 720, 480, 360)"
            );
            exit(0);
        }
        "-b" | "--debug" => todo = "debug",
        "-g" | "--get" => todo = "print link",
        "-d" | "--download" => {
            todo!();
        }
        "-D" | "--dub" => mode = "dub",
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

    allanime(&query, todo, mode, &provider, &quality).await;
}
