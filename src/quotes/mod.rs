use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quote {
    pub category: String,
    pub text: String,
}

lazy_static! {
    static ref ALL_QUOTES: Vec<Quote> = load_quotes();
}

fn load_quotes() -> Vec<Quote> {
    println!("Current working directory: {:?}", env::current_dir());
    let path = if cfg!(debug_assertions) {
        "src/quotes/quotes.json" // Local path
    } else {
        "/app/src/quotes/quotes.json" // Deployed path
    };
    println!("Attempting to read file at: {}", path);
    let file_content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("Could not read {}: {}", path, e));
    serde_json::from_str(&file_content).expect("Could not parse quotes.json")
}

pub async fn get_random_quote() -> Quote {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..ALL_QUOTES.len());

    // Iterate the existing quotes into a new vector
    let mut new_quotes = Vec::new();
    for quote in ALL_QUOTES.iter() {
        new_quotes.push(quote.clone());
    }

    new_quotes[index].clone()
}
