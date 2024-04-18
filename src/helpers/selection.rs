use skim::prelude::{Key::ESC, Skim, SkimItemReader, SkimOptionsBuilder};
use std::{
    error::Error,
    io::{Cursor, Read, Write},
    process::{exit, Command, Stdio},
};

fn skim(selection: &str, prompt: &str, is_multi: bool) -> String {
    let options = SkimOptionsBuilder::default()
        //.height(Some("33%"))
        .reverse(true)
        .multi(is_multi)
        .nosort(true)
        .prompt(Some(prompt))
        .build()
        .unwrap_or_else(|_| {
            eprintln!("Failed to build options for fuzzy selector skim");
            exit(1);
        });

    let items = SkimItemReader::default().of_bufread(Cursor::new(selection.to_owned()));

    Skim::run_with(&options, Some(items))
        .map(|out| {
            if out.final_key == ESC {
                eprintln!("Nothing's selected");
                exit(0);
            }

            out.selected_items
                .iter()
                .map(|item| item.output())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .expect("No input to fuzzy selector skim")
}

pub fn selection(selection: &str, prompt: &str, is_multi: bool, is_rofi: bool) -> String {
    if is_rofi {
        rofi(selection, prompt, is_multi).expect("Failed to use rofi")
    } else {
        skim(selection, prompt, is_multi)
    }
}

fn rofi(selection: &str, prompt: &str, is_multi: bool) -> Result<String, Box<dyn Error>> {
    let multi = if is_multi { "-multi-select" } else { "" };

    let process = Command::new("rofi")
        .arg("-dmenu")
        .arg("-i")
        .arg(multi)
        .args(["-p", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute rofi");

    process.stdin.unwrap().write_all(selection.as_bytes())?;

    let mut output = String::new();

    match process.stdout.unwrap().read_to_string(&mut output) {
        Ok(_) => Ok(output.trim_end_matches('\n').to_owned()),
        Err(err) => Err(Box::new(err)),
    }
}
