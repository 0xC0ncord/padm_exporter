use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, TextEncoder, gather};
use std::convert::Infallible;
use std::sync::Arc;
use std::thread;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

use crate::client::PADMClient;
use crate::config;
use crate::metrics::MetricsRegistry;

async fn metrics_handler(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(Response::new(Full::new(Bytes::from(buffer))))
}

pub async fn run(config: config::Config) -> Result<()> {
    let listener = TcpListener::bind(config.bind_address()).await?;

    let registry = Arc::new(MetricsRegistry::new());

    // Spawn client threads
    for target in config.targets() {
        let mut client = PADMClient::new(
            target.addr(),
            target.url(),
            target.tls_insecure(),
            target.interval(),
            target.username(),
            target.password(),
            registry.clone(),
        );
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move { client.run(thread::current()).await });
            loop {
                thread::park();
            }
        });
    }

    // Start metrics service
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
