use actix_web::{web::{self, Data}, App, HttpServer, HttpResponse};
use std::sync::Mutex;
use std::time::Duration;
use std::thread;

use crate::config;
use crate::server;

async fn index(body_mutex: Data<Mutex<String>>) -> HttpResponse {
    // Wait until we have data
    while *body_mutex.lock().unwrap() == "" {
        async_std::task::sleep(Duration::from_millis(100)).await;
    }
    HttpResponse::Ok().body(format!("{}", *body_mutex.lock().unwrap()))
}

pub async fn run(config: &config::Config) -> std::io::Result<()> {
    // Create global body reference
    let body_mutex = Data::new(Mutex::new(String::from("")));
    let body_mutex_copy = body_mutex.clone();

    let config_copy = config.clone();

    // Spawn probe thread
    thread::spawn(move || {
        server::probe::run(&config_copy, body_mutex_copy)
    });

    // Startup
    HttpServer::new(move || {
        App::new()
            .app_data(body_mutex.clone())
            .route("/padm", web::get().to(index))
        })
        .bind(&config.bind_address())?
        .run()
        .await
}
