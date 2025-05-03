use actix_web::{get, web::ServiceConfig, HttpResponse, Responder};

#[get("/")]
pub async fn hello_world() -> impl Responder {
    // Logic here

    HttpResponse::Ok().body("Hello World fuceewfwek")
}
