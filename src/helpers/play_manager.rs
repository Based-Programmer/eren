use crate::{Todo, Vid};
use std::{
    env, fs,
    process::{Command, Stdio},
};

pub async fn play_manage(mut vid: Vid, todo: Todo) {
    match todo {
        Todo::Play => {
            let mut mpv_args = Vec::new();

            if let Some(audio_link) = vid.audio_link {
                mpv_args.push(format!("--audio-file={}", audio_link))
            }

            if let Some(sub_link) = vid.subtitle_link {
                mpv_args.push(format!("--sub-file={}", sub_link))
            }

            if let Some(referrer) = vid.referrer {
                mpv_args.push(format!("--referrer={referrer}"));
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
                .args(mpv_args)
                .args([
                    &vid.vid_link,
                    "--no-terminal",
                    "--force-seekable",
                    "--force-window=immediate",
                    "--speed=1",
                    "--sub-visibility",
                    &format!("--force-media-title={}", vid.title),
                ])
                //.arg(format!("--user-agent={}", vid.user_agent))
                //.arg(format!("--referrer={}", vid.referrer))
                .output()
                .expect("Failed to execute mpv")
                .status
                .code()
                .unwrap()
                == 2
            {
                eprintln!("Faulty video link");
            }
        }
        Todo::Download => {
            vid.title = vid.title.replace(" /", "").replace('/', "");

            if vid.vid_link.ends_with(".m3u8") {
                if Command::new("hls")
                    .args(["-n", "38"])
                    .args(["-o", &vid.title])
                    .arg(&vid.vid_link)
                    .status()
                    .expect("Failed to execute hls")
                    .success()
                {
                    println!("\nDownload Completed: {}", vid.title);
                } else {
                    eprintln!("\nDownload failed {}", vid.title);
                }
            } else if let Some(audio_link) = &vid.audio_link {
                download(&vid, &vid.vid_link, " video", "mp4").await;

                download(&vid, audio_link, " audio", "mp3").await;

                let vid_title = format!("{} video.{}", vid.title, "mp4");
                let audio_title = format!("{} audio.{}", vid.title, "mp3");

                if Command::new("ffmpeg")
                    .args(["-i", &vid_title])
                    .args(["-i", &audio_title])
                    .args(["-c", "copy"])
                    .arg(format!("{}.mp4", vid.title))
                    .status()
                    .expect("Failed to execute ffmpeg")
                    .success()
                {
                    println!("\nVideo & audio merged successfully");

                    fs::remove_file(vid_title)
                        .unwrap_or_else(|_| eprintln!("Failed to remove downloaded video"));

                    fs::remove_file(audio_title)
                        .unwrap_or_else(|_| eprintln!("Failed to remove downloaded audio"));
                } else {
                    eprintln!("\nVideo & audio merge failed");
                }
            } else {
                download(&vid, &vid.vid_link, "", "mp4").await;
            }

            if let Some(sub_link) = &vid.subtitle_link {
                download(&vid, sub_link, " subtitle", "srt").await;
            }
        }
        Todo::GetLink => {
            let mut vid_link_printed = false;

            if let Some(audio_link) = vid.audio_link {
                println!("\n{}", vid.vid_link);
                println!("{}", audio_link);
                vid_link_printed = true;
            }

            if let Some(sub_link) = vid.subtitle_link {
                if !vid_link_printed {
                    println!("\n{}", vid.vid_link);
                }
                println!("{}", sub_link);
                vid_link_printed = true;
            }

            if !vid_link_printed {
                println!("{}", vid.vid_link);
            }
        }
        Todo::Debug => println!("{vid:#?}"),
    }
}

async fn download(vid: &Vid, link: &str, types: &str, extension: &str) {
    println!("\nDownloading{}: {}", types, vid.title);

    let mut aria_args = vec![format!("--out={}{}.{}", vid.title, types, extension)];

    if let Some(referer) = vid.referrer {
        aria_args.push(format!("--referer={referer}"));
    }

    if Command::new("aria2c")
        .args(aria_args)
        .args([
            link,
            "--max-connection-per-server=16",
            "--max-concurrent-downloads=16",
            "--split=16",
            "--min-split-size=1M",
            "--check-certificate=false",
            "--summary-interval=0",
            "--download-result=hide",
        ])
        //.arg(format!("--user-agent={}", vid.user_agent))
        .status()
        .expect("Failed to execute aria2c")
        .success()
    {
        println!("\nDownloaded successfully");
    } else {
        eprintln!("\nDownload Failed");
    }
}
