use crate::Vid;
use std::{
    env,
    process::{Command, Stdio},
};

pub async fn play_manage(vid: Vid, todo: &str) {
    match todo {
        "debug" => println!("{vid:#?}"),
        "print link" => {
            println!("{}", vid.vid_link);

            if !vid.audio_link.is_empty() {
                println!("{}", vid.audio_link);
            }

            if !vid.subtitle_link.is_empty() {
                println!("{}", vid.subtitle_link);
            }
        }
        "play" => {
            let mut audio_arg = String::new();
            let mut sub_arg = String::new();

            if !vid.audio_link.is_empty() {
                audio_arg = format!("--audio-file={}", vid.audio_link)
            }

            if !vid.subtitle_link.is_empty() {
                sub_arg = format!("--sub-file={}", vid.subtitle_link)
            }

            if env::consts::OS == "android" {
                Command::new("am")
                    .arg("start")
                    .args(["--user", "0"])
                    .args(["-a", "android.intent.action.VIEW"])
                    .args(["-d", &vid.vid_link])
                    .args(["-n", "is.xyz.mpv/.MPVActivity"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to execute am command");
            } else {
                Command::new("mpv")
                    .arg(vid.vid_link)
                    .arg(audio_arg)
                    .arg(sub_arg)
                    .arg("--no-terminal")
                    .arg("--force-window=immediate")
                    .arg("--speed=1")
                    .arg(format!("--force-media-title={}", vid.title))
                    .arg(format!("--user-agent={}", vid.user_agent))
                    .arg(format!("--referrer={}", vid.referrer))
                    .spawn()
                    .expect("Failed to execute mpv");
            }
        }
        _ => {}
    }
}
