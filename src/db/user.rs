use crate::models::user::{NewUser, User};
use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::PgPool;

#[get("/user/{id}")]
pub async fn user(pool: web::Data<PgPool>, id: web::Path<i32>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id.into_inner())
        .fetch_one(pool.get_ref())
        .await;

    match user {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[post("/user")]
pub async fn add_user(pool: web::Data<PgPool>, new_user: web::Json<NewUser>) -> impl Responder {
    // In production, hash the password before storing!
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *",
    )
    .bind(&new_user.email)
    .bind(&new_user.password) // REMEMBER: Hash this in production!
    .fetch_one(pool.get_ref())
    .await;

    match user {
        Ok(user) => HttpResponse::Created().json(user),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
