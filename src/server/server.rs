use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::config;
use crate::server;

async fn index(body_mutex: Data<Arc<Mutex<String>>>) -> HttpResponse {
    // Wait until we have data
    if (*body_mutex.lock().unwrap()).is_empty() {
        async_std::task::sleep(Duration::from_millis(1000)).await;
    }
    HttpResponse::Ok().body((*body_mutex.lock().unwrap()).to_string())
}

pub async fn run(config: config::Config) -> std::io::Result<()> {
    // Create global body reference
    let body_mutex = Arc::new(Mutex::new(String::new()));
    let body_mutex_clone = body_mutex.clone();
    let bind_address = config.bind_address();

    // Spawn probe thread
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move { server::probe::run(config, body_mutex_clone).await });
        loop {
            thread::park();
        }
    });

    // Startup
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(body_mutex.clone()))
            .route("/padm", web::get().to(index))
    })
    .bind(bind_address)?
    .run()
    .await
}
