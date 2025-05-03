use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug)] // Add Debug
pub struct User {
    pub user: Uuid,
}

#[post("/users")]
pub async fn create_user(
    supabase: web::Data<SupabaseConfig>,
    user: web::Json<User>,
) -> HttpResponse {
    // Validate and trim URL
    let url = format!("{}/rest/v1/users", supabase.url.trim());
    if !url.starts_with("https://") {
        eprintln!("Invalid Supabase URL: {}", url);
        return HttpResponse::InternalServerError().body("Invalid Supabase URL");
    }

    // Log debugging information
    println!("Request URL: {}", url);
    println!("Supabase Key: {}", supabase.key);
    println!("Request Body: {:?}", user);

    let client = reqwest::Client::new();
    let response = match client
        .post(&url)
        .header("apikey", &supabase.key)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&user.into_inner())
        .send()
        .await
    {
        Ok(res) => {
            println!("Supabase response status: {}", res.status());
            res
        }
        Err(err) => {
            eprintln!("Supabase request error: {:?}", err);
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    match response.json::<serde_json::Value>().await {
        Ok(json) => {
            println!("Supabase response JSON: {:?}", json);
            HttpResponse::Ok().json(json)
        }
        Err(err) => {
            eprintln!("Supabase JSON parse error: {:?}", err);
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}
