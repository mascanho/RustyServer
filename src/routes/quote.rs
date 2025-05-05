use actix_web::http::StatusCode as ActixStatusCode;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
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

#[get("/random")]
pub async fn gen_quote() -> impl Responder {
    let quote = quotes::get_random_quote().await;
    HttpResponse::Ok().json(quote)
}

#[post("/add")]
pub async fn add_quote(
    supabase: web::Data<SupabaseConfig>,
    body: web::Json<IncomingQuote>,
) -> HttpResponse {
    let url = format!("{}/rest/v1/quotes", supabase.url.trim());
    if !url.starts_with("https://") {
        eprintln!("Invalid Supabase URL: {}", url);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Invalid database URL ",
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
        .json(&quote_data) // Fixed: Use quote_data instead of "e_data
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Supabase request error: {:?}", err);
            return HttpResponse::BadGateway().json(json!({
                "status": "error",
                "message": "Failed to communicate with database ",
                "details": err.to_string()
            }));
        }
    };

    let status = ActixStatusCode::from_u16(response.status().as_u16())
        .unwrap_or(ActixStatusCode::INTERNAL_SERVER_ERROR);

    if !status.is_success() {
        let error_text = match response.text().await {
            Ok(text) => text,
            Err(err) => format!("Failed to read error response: {}", err),
        };

        return HttpResponse::build(status).json(json!({
            "status": "error",
            "message": "database request failed ",
            "details": error_text,
            "status_code": status.as_u16()
        }));
    }

    match response.json::<serde_json::Value>().await {
        Ok(json) => {
            println!("RustySEO response: {:?}", json);
            HttpResponse::Ok().json(json!({
                "message": "Thank you dude! Your wisdom has been preserved for eternity! 🤘",
                "details": {
                    "your_quote": {
                        "category": quote_data.category,
                        "text": quote_data.quote
                    },
                    "supabase_id": json.get("id").unwrap_or(&serde_json::Value::Null),
                    "timestamp": format!("{}", OffsetDateTime::now_utc())
                },
                "fun_fact": "The word \"dude\" first appeared in 1883 as a term for a fastidious man!"
            }))
        }
        Err(err) => {
            eprintln!("Failed to parse Supabase response: {:?}", err);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Unexpected response format from Supabase ",
                "details": err.to_string()
            }));
        }
    }
}
