use std::collections::HashMap;
use std::sync::Mutex;

use actix_web::middleware::Logger;
use actix_web::{get, http::header, post, web, App, HttpResponse, HttpServer, Responder};
use rand::prelude::IteratorRandom;

struct AppState {
    urls: Mutex<HashMap<String, String>>,
}

const BASE_URL: &str = "http://localhost:8080/";
const AVAILABLE_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const SHORT_ID_LEN: usize = 6;

/// Generate a short ID
fn generate_short_id() -> String {
    let mut rng = rand::thread_rng();
    let mut result = String::with_capacity(SHORT_ID_LEN);
    for _ in 0..SHORT_ID_LEN {
        let picked_char = AVAILABLE_CHARS.chars().choose(&mut rng).unwrap();
        result.push(picked_char)
    }
    result
}

#[get("/{url}")]
async fn url_redirection(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let short_url = path.into_inner();
    let urls = data.urls.lock().unwrap();
    match urls.get(&short_url) {
        Some(long_url) => HttpResponse::MovedPermanently()
            .insert_header((header::LOCATION, long_url.clone()))
            .finish(),
        None => HttpResponse::NotFound().finish(),
    }
}

#[post("/")]
async fn register(req_body: String, data: web::Data<AppState>) -> impl Responder {
    let mut urls = data.urls.lock().unwrap();
    match urls.get(&req_body) {
        Some(value) => HttpResponse::Ok().body(value.clone()),
        None => {
            let mut url = String::from(BASE_URL);
            let short_id = generate_short_id();
            url.push_str(&short_id);
            urls.insert(short_id.clone(), req_body);
            HttpResponse::Ok().body(short_id)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let port_str = std::env::var("PORT").unwrap_or("8080".to_string());
    let port: u16 = port_str.parse().expect("invalid port");

    let state = web::Data::new(AppState {
        urls: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%s %r in %Dms"))
            .app_data(state.clone())
            .service(url_redirection)
            .service(register)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
