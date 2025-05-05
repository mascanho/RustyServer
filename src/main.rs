use actix_web::{web, web::ServiceConfig};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;

mod routes;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let supabase_url = secrets.get("SUPABASE_URL").expect("Missing SUPABASE_URL");
    let supabase_key = secrets.get("SUPABASE_KEY").expect("Missing SUPABASE_KEY");

    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(web::Data::new(routes::user::SupabaseConfig {
            url: supabase_url,
            key: supabase_key,
        }));
        cfg.service(routes::index::redirect);
        cfg.service(routes::user::create_user);
    };

    Ok(config.into())
}
