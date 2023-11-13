use skim::prelude::{Skim, SkimItemReader, SkimOptionsBuilder};
use std::fmt::Write as _;
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

    let items = SkimItemReader::default().of_bufread(Cursor::new(selection.to_string()));

    Skim::run_with(&options, Some(items))
        .map(|out| {
            out.selected_items
                .iter()
                .fold(String::new(), |mut output, item| {
                    let _ = writeln!(output, "{}", item.output());
                    output
                })
                .trim_end_matches('\n')
                .to_string()
        })
        .unwrap_or_else(|| {
            eprintln!("No input to fuzzy selector skim");
            exit(1);
        })
}

pub fn selection(selection: &str, prompt: &str, is_multi: bool, is_not_rofi: bool) -> String {
    if is_not_rofi {
        skim(selection, prompt, is_multi)
    } else {
        match rofi(selection, prompt, is_multi) {
            Ok(selection) => selection,
            Err(err) => {
                eprintln!("Error: {err}");
                exit(1);
            }
        }
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
        .spawn()?;

    process.stdin.unwrap().write_all(selection.as_bytes())?;

    let mut output = String::new();

    match process.stdout.unwrap().read_to_string(&mut output) {
        Ok(_) => Ok(output.trim_end_matches('\n').to_string()),
        Err(err) => Err(Box::new(err)),
    }
}
