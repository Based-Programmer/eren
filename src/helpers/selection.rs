use skim::prelude::*;
use std::io::Cursor;

pub fn selection(selecttion: &str, prompt: &str) -> String {
    let options = SkimOptionsBuilder::default()
        //.height(Some("33%"))
        .reverse(true)
        //.multi(true)
        .prompt(Some(prompt))
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(selecttion.to_string()));

    // `run_with` would read and show items from the stream
    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(Vec::new);

    selected_items
        .iter()
        .map(|item| item.output())
        .collect::<String>()
}
