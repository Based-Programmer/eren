use crate::Vid;
use std::{env, process::Stdio, sync::Arc};
use tokio::{fs, process::Command, sync::Mutex};

pub async fn play_manage(vid: Vid, todo: Arc<Mutex<&String>>) {
    let todo = todo.lock().await;

    match todo.as_str() {
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
            } else if Command::new("mpv")
                .arg(vid.vid_link)
                .arg(audio_arg)
                .arg(sub_arg)
                .arg("--no-terminal")
                .arg("--force-window=immediate")
                .arg("--speed=1")
                .arg("--sub-visibility")
                .arg(format!("--force-media-title={}", vid.title))
                .arg(format!("--user-agent={}", vid.user_agent))
                .arg(format!("--referrer={}", vid.referrer))
                .output()
                .await
                .expect("Failed to execute mpv")
                .status
                .code()
                .unwrap()
                == 2
            {
                eprintln!("Faulty video link");
            }
        }
        "download" => {
            if vid.vid_link.ends_with(".m3u8") {
                if Command::new("hls")
                    .args(["-n", "38"])
                    .args(["-o", &vid.title])
                    .arg(&vid.vid_link)
                    .status()
                    .await
                    .expect("Failed to execute hls")
                    .success()
                {
                    println!("\nDownload Completed: {}", vid.title);
                } else {
                    eprintln!("\nDownload failed {}", vid.title);
                }
            } else if !vid.audio_link.is_empty() {
                download(&vid, &vid.vid_link, " video", "mp4").await;

                download(&vid, &vid.audio_link, " audio", "mp3").await;

                let vid_title = format!("{} video.{}", vid.title, "mp4");
                let audio_title = format!("{} audio.{}", vid.title, "mp3");

                if Command::new("ffmpeg")
                    .args(["-i", &vid_title])
                    .args(["-i", &audio_title])
                    .args(["-c", "copy"])
                    .arg(format!("{}.mp4", vid.title))
                    .status()
                    .await
                    .expect("Failed to execute ffmpeg")
                    .success()
                {
                    println!("\nVideo & audio merged successfully");

                    fs::remove_file(vid_title)
                        .await
                        .unwrap_or_else(|_| eprintln!("Failed to remove downloaded video"));

                    fs::remove_file(audio_title)
                        .await
                        .unwrap_or_else(|_| eprintln!("Failed to remove downloaded audio"));
                } else {
                    eprintln!("\nVideo & audio merge failed");
                }
            } else {
                download(&vid, &vid.vid_link, "", "mp4").await;
            }

            if !vid.subtitle_link.is_empty() {
                download(&vid, &vid.subtitle_link, " subtitle", "srt").await;
            }
        }
        _ => {}
    }
}

async fn download(vid: &Vid, link: &str, types: &str, extension: &str) {
    println!("\nDownloading{}: {}", types, vid.title);

    if Command::new("aria2c")
        .arg(link)
        .arg("--max-connection-per-server=16")
        .arg("--max-concurrent-downloads=16")
        .arg("--split=16")
        .arg("--min-split-size=1M")
        .arg("--check-certificate=false")
        .arg("--summary-interval=0")
        .arg("--download-result=hide")
        .arg(format!("--out={}{}.{}", vid.title, types, extension))
        .arg(format!("--user-agent={}", vid.user_agent))
        .arg(format!("--referer={}", vid.referrer))
        .status()
        .await
        .expect("Failed to execute aria2c")
        .success()
    {
        println!("\nDownloaded successfully");
    } else {
        eprintln!("\nDownload Failed");
    }
}
