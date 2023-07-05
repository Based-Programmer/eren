use skim::prelude::*;
use std::{
    error::Error,
    io::{
        prelude::{Read, Write},
        Cursor,
    },
    process::{Command, Stdio},
};

fn skim(selecttion: &str, prompt: &str, is_multi: bool) -> String {
    let options = SkimOptionsBuilder::default()
        //.height(Some("33%"))
        .reverse(true)
        .multi(is_multi)
        .prompt(Some(prompt))
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(selecttion.to_string()));

    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(Vec::new);

    selected_items
        .iter()
        .map(|item| format!("{}\n", item.output()))
        .collect::<String>()
        .trim_end_matches('\n')
        .to_string()
}

pub fn selection(selecttion: &str, prompt: &str, is_multi: bool, is_not_rofi: bool) -> String {
    if is_not_rofi {
        skim(selecttion, prompt, is_multi)
    } else {
        rofi(selecttion, prompt, is_multi).unwrap()
    }
}

fn rofi(selecttion: &str, prompt: &str, is_multi: bool) -> Result<String, Box<dyn Error>> {
    let multi = if is_multi { "-multi-select" } else { "" };

    let process = Command::new("rofi")
        .arg("-dmenu")
        .arg("-i")
        .arg(multi)
        .args(["-p", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    process
        .stdin
        .unwrap()
        .write_all(selecttion.as_bytes())
        .unwrap();

    let mut output = String::new();

    match process.stdout.unwrap().read_to_string(&mut output) {
        Ok(_) => Ok(output.trim_end_matches('\n').to_string()),
        Err(err) => Err(Box::new(err)),
    }
}
