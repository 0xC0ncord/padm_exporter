use anyhow::{Context, Result};
use http_body_util::{BodyExt, combinators::BoxBody};
use http_body_util::{Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, TextEncoder};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use url::form_urlencoded;

use crate::client::PADMClient;
use crate::config;

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn metrics_handler(
    req: Request<hyper::body::Incoming>,
    clients: HashMap<String, Arc<PADMClient>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
    // Only allow GET requests
    if req.method() != Method::GET {
        let mut not_allowed = Response::new(empty());
        *not_allowed.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(not_allowed);
    }
    // Only allow to the /padm path
    if req.uri().path() != "/padm" {
        let mut not_found = Response::new(empty());
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        return Ok(not_found);
    }
    // Extract the target parameter
    let query = req.uri().query().unwrap_or("");
    let target = match form_urlencoded::parse(query.as_bytes())
        .find(|(key, _)| key == "target")
        .map(|(_, value)| value.to_string())
    {
        Some(t) => t,
        None => {
            // Return 400 if not given
            let mut bad_request = Response::new(empty());
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(bad_request);
        }
    };

    let client = clients.get(&target).context("Requested target {} not found.")?;

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    // Get all metrics from the registry
    let metrics = client.registry().read().await.registry.gather();
    encoder.encode(&metrics, &mut buffer).unwrap();

    // Send it
    Ok(Response::new(full(Bytes::from(buffer)).boxed()))
}

pub async fn run(config: config::Config) -> Result<()> {
    let listener = TcpListener::bind(config.bind_address()).await?;

    let mut clients = HashMap::new();

    // Spawn client threads
    for target in config.targets() {
        let client = PADMClient::new(
            target.addr(),
            target.url(),
            target.tls_insecure(),
            target.interval(),
            target.username(),
            target.password(),
        );
        let client_arc = Arc::new(client);
        clients.insert(target.host().to_string(), client_arc.clone());

        let client_thread_arc = client_arc.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                client_thread_arc.run(thread::current()).await
            });
            loop {
                thread::park();
            }
        });
    }

    // Start metrics service
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let clients_clone = clients.clone();

        tokio::task::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(
                    io,
                    hyper::service::service_fn(move |req| {
                        metrics_handler(req, clients_clone.clone())
                    }),
                )
                .await
            {
                log::error!("Error serving connection: {e:?}");
            }
        });
    }
}
