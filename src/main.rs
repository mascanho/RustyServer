use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder, PeerIpKeyExtractor};
use actix_web::{web, web::ServiceConfig};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;

mod quotes;
mod routes;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let supabase_url = secrets.get("SUPABASE_URL").expect("Missing SUPABASE_URL");
    let supabase_key = secrets.get("SUPABASE_KEY").expect("Missing SUPABASE_KEY");

    // Configure in-memory rate limiting for quote endpoints
    let governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(1) // 1 request per second (60 req/min)
        .burst_size(10) // Allow bursts of 10 requests
        .key_extractor(PeerIpKeyExtractor)
        .finish()
        .unwrap();

    let supabase_url = supabase_url.clone();
    let supabase_key = supabase_key.clone();

    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(web::Data::new(routes::user::SupabaseConfig {
            url: supabase_url,
            key: supabase_key,
        }));

        // Unaffected routes
        cfg.service(routes::index::redirect);
        cfg.service(routes::user::create_user);

        // Rate-limited quote endpoints with CORS and logging
        cfg.service(
            web::scope("/api/quote")
                .wrap(
                    Cors::default()
                        .allow_any_origin() // For testing; restrict in production
                        .allowed_methods(vec!["GET", "POST"])
                        .allow_any_header()
                        .max_age(3600),
                )
                .wrap(actix_web::middleware::Logger::default()) // For debugging
                .wrap(Governor::new(&governor_conf))
                .service(routes::quote::gen_quote)
                .service(routes::quote::add_quote),
        );
    };

    Ok(config.into())
}
