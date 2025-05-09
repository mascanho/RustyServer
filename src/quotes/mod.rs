use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quote {
    pub category: String,
    pub text: String,
}

const QUOTES_JSON: &str = include_str!("quotes.json");

lazy_static! {
    static ref ALL_QUOTES: Vec<Quote> =
        serde_json::from_str(QUOTES_JSON).expect("Could not parse embedded quotes.json");
}

pub async fn get_random_quote() -> Quote {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..ALL_QUOTES.len());
    ALL_QUOTES[index].clone()
}
