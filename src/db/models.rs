pub mod models;
pub mod repository;

use mongodb::{Client, Database};
use std::error::Error;

pub async fn connect(uri: &str) -> Result<Database, Box<dyn Error>> {
    let client_options = ClientOptions::parse(uri).await?;
    let client = Client::with_options(client_options)?;

    // Ping the server to verify connection
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;

    println!("Successfully connected to MongoDB!");

    Ok(client.database("actix_db"))
}
