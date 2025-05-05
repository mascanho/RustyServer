use actix_web::{get, web::ServiceConfig, HttpResponse, Responder};

#[get("/")]
pub async fn redirect() -> impl Responder {
    // Logic here

    HttpResponse::Found()
        .append_header(("Location", "https://rustyseo.com"))
        .finish()
}
