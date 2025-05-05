use actix_web::http::StatusCode as ActixStatusCode;
use actix_web::{get, post, web, HttpResponse, Responder};
use reqwest::StatusCode as ReqwestStatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::quotes;
use crate::routes::user::SupabaseConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct Quote {
    pub id: Option<Uuid>,
    pub category: String,
    pub quote: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct IncomingQuote {
    pub category: String,
    pub quote: String,
}

#[get("/api/quote/random")]
pub async fn gen_quote() -> impl Responder {
    let quote = quotes::get_random_quote().await;
    HttpResponse::Ok().json(quote)
}

#[post("/api/quote/add")]
pub async fn add_quote(
    supabase: web::Data<SupabaseConfig>,
    body: web::Json<IncomingQuote>,
) -> HttpResponse {
    // Validate and trim URL
    let url = format!("{}/rest/v1/quotes", supabase.url.trim());
    if !url.starts_with("https://") {
        eprintln!("Invalid Supabase URL: {}", url);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Invalid Supabase URL",
            "details": "URL must start with https://"
        }));
    }

    let quote_data = body.into_inner();

    println!("Request URL: {}", url);
    println!("Request Body: {:?}", quote_data);

    let client = reqwest::Client::new();
    let response = match client
        .post(&url)
        .header("apikey", &supabase.key)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&quote_data)
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Supabase request error: {:?}", err);
            return HttpResponse::BadGateway().json(json!({
                "status": "error",
                "message": "Failed to communicate with DB 😭",
                "details": err.to_string()
            }));
        }
    };

    // Convert reqwest status to actix status
    let status = ActixStatusCode::from_u16(response.status().as_u16())
        .unwrap_or(ActixStatusCode::INTERNAL_SERVER_ERROR);

    if !status.is_success() {
        let error_text = match response.text().await {
            Ok(text) => text,
            Err(err) => format!("Failed to read error response: {}", err),
        };

        return HttpResponse::build(status).json(json!({
            "status": "error",
            "message": "Database request failed 😨",
            "details": error_text,
            "status_code": status.as_u16()
        }));
    }

    match response.json::<serde_json::Value>().await {
        Ok(json) => {
            println!("Supabase response: {:?}", json);
            HttpResponse::Ok().json(json!({
                "message": "Thank you dude! Your wisdom has been preserved for eternity! 🤘",
                "details": {
                    "your_quote": {
                        "category": quote_data.category,
                        "text": quote_data.quote
                    },
                    "supabase_id": json.get("id").unwrap_or(&serde_json::Value::Null),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                },
                "RustySEO": "A next generation SEO/GEO toolkit for Marketers and Developers"
            }))
        }
        Err(err) => {
            eprintln!("Failed to parse Supabase response: {:?}", err);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Unexpected response format from Supabase",
                "details": err.to_string()
            }))
        }
    }
}
