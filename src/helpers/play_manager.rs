use crate::{Todo, Vid, RED, RESET, YELLOW};
use std::{
    env::consts::OS,
    fs,
    process::{Command, Stdio},
};

pub async fn play_manage(mut vid: Vid, todo: Todo) {
    match todo {
        Todo::Play => {
            if OS == "android" {
                Command::new("am")
                    .arg("start")
                    .args(["--user", "0"])
                    .args(["-a", "android.intent.action.VIEW"])
                    .args(["-d", &vid.vid_link])
                    .args(["-n", "is.xyz.mpv/.MPVActivity"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to execute 'am' cmd");
            } else {
                let mut mpv_args = Vec::new();

                if let Some(audio_link) = vid.audio_link {
                    mpv_args.push(format!("--audio-file={}", audio_link))
                }

                if let Some(sub_file) = vid.subtitle_path {
                    mpv_args.push(format!("--sub-file={}", sub_file));
                    mpv_args.push(String::from("--sub-visibility"))
                }

                if let Some(referrer) = vid.referrer {
                    mpv_args.push(format!("--referrer={}", referrer))
                }

                Command::new("clear")
                    .spawn()
                    .expect("Failed to execute 'clear' cmd");

                Command::new("mpv")
                    .args(mpv_args)
                    .args([
                        &vid.vid_link,
                        "--force-seekable",
                        "--force-window=immediate",
                        "--speed=1",
                        &format!("--force-media-title={}", vid.title),
                    ])
                    .stdout(Stdio::null())
                    .status()
                    .expect("Failed to execute mpv");
            }
        }
        Todo::Download => {
            vid.title = vid.title.replace('/', "\\");

            if vid.vid_link.ends_with(".m3u8") {
                if Command::new("hls")
                    .args(["-n", "38"])
                    .args(["-o", &vid.title])
                    .arg(&vid.vid_link)
                    .status()
                    .expect("Failed to execute hls
                        Copy the script from https://github.com/CoolnsX/hls_downloader/blob/main/hls &
                        move it to your $PATH")
                    .success()
                {
                    println!("{}\nDownload Completed: {}{}", YELLOW, vid.title, RESET);
                } else {
                    eprintln!("{}\nDownload failed: {}{}", RED, vid.title, RESET);
                }
            } else if let Some(audio_link) = &vid.audio_link {
                download(&vid, &vid.vid_link, " video", "mp4").await;
                download(&vid, audio_link, " audio", "mp3").await;

                let vid_title = format!("{} video.{}", vid.title, "mp4");
                let audio_title = format!("{} audio.{}", vid.title, "mp3");
                let mut ffmpeg_args = vec!["-i", &vid_title, "-i", &audio_title];

                let vid_ext = if let Some(sub_file) = &vid.subtitle_path {
                    ffmpeg_args.extend_from_slice(&["-i", sub_file, "-c:s", "ass"]);
                    "mkv"
                } else {
                    "mp4"
                };

                ffmpeg_args.extend_from_slice(&[
                    "-map", "0:v", "-map", "1:a", "-map", "2:s", "-c:v", "copy", "-c:a", "copy",
                ]);

                if Command::new("ffmpeg")
                    .args(ffmpeg_args)
                    .arg(format!("{}.{}", vid.title, vid_ext))
                    .output()
                    .expect("Failed to execute ffmpeg")
                    .status
                    .success()
                {
                    println!("{YELLOW}Video & audio merged successfully{RESET}");

                    fs::remove_file(vid_title).unwrap_or_else(|_| {
                        eprintln!("{RED}Failed to remove downloaded video{RESET}")
                    });

                    fs::remove_file(audio_title).unwrap_or_else(|_| {
                        eprintln!("{RED}Failed to remove downloaded audio{RESET}")
                    });
                } else {
                    eprintln!("{RED}Video & audio merge failed{RESET}");
                }
            } else {
                download(&vid, &vid.vid_link, "", "mp4").await;
            }
        }
        Todo::GetLink => {
            let mut vid_link_printed = false;

            if let Some(audio_link) = vid.audio_link {
                println!("\n{}", vid.vid_link);
                println!("{}", audio_link);
                vid_link_printed = true;
            }

            if let Some(sub_file) = vid.subtitle_path {
                if !vid_link_printed {
                    println!("\n{}", vid.vid_link);
                }
                println!("{}", sub_file);
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
    println!("{}\nDownloading{}:{} {}", YELLOW, types, RESET, vid.title);

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
        println!("{YELLOW}\nDownloaded successfully{RESET}");
    } else {
        eprintln!("{RED}\nDownload Failed{RESET}");
    }
}
