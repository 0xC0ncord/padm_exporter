use anyhow::Result;
use std::convert::Infallible;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use prometheus::{Encoder, TextEncoder, gather};
use std::thread;
use tokio::runtime::Runtime;

use crate::server;
use crate::config;

async fn metrics_handler(_req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(Response::new(Full::new(Bytes::from(buffer))))
}

pub async fn run(config: config::Config) -> Result<()> {
    let listener = TcpListener::bind(config.bind_address()).await?;

    // Spawn probe thread
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move { server::probe::run(config).await });
        loop {
            thread::park();
        }
    });

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, hyper::service::service_fn(metrics_handler))
                .await
            {
                log::error!("Error serving connection: {e:?}");
            }
        });
    }
}
