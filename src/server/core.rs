use anyhow::Result;
use http_body_util::{BodyExt, combinators::BoxBody};
use http_body_util::{Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, TextEncoder, proto::MetricFamily};
use std::sync::Arc;
use std::thread;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use url::form_urlencoded;

use crate::client::PADMClient;
use crate::config;
use crate::metrics::MetricsRegistry;

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
    registry: Arc<MetricsRegistry>,
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

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    // Get all metric families from the registry
    let all_families = registry.registry.gather();
    // Filter down metrics to only the target given
    let filtered_families: Vec<MetricFamily> = all_families
        .into_iter()
        .filter_map(|mut family| {
            let original_metrics = family.get_metric().to_vec();
            let filtered_metrics: Vec<_> = original_metrics
                .into_iter()
                .filter(|m| {
                    m.get_label()
                        .iter()
                        .any(|l| l.name() == "target" && l.value() == target)
                })
                .collect();
            if filtered_metrics.is_empty() {
                None
            } else {
                family.metric.clear();
                family.mut_metric().extend(filtered_metrics);
                Some(family)
            }
        })
        .collect();
    encoder.encode(&filtered_families, &mut buffer).unwrap();

    // Send it
    Ok(Response::new(full(Bytes::from(buffer)).boxed()))
}

pub async fn run(config: config::Config) -> Result<()> {
    let listener = TcpListener::bind(config.bind_address()).await?;

    let registry = Arc::new(MetricsRegistry::new());

    // Spawn client threads
    for target in config.targets() {
        let mut client = PADMClient::new(
            target.host(),
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

        let registry_clone = registry.clone();

        tokio::task::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(
                    io,
                    hyper::service::service_fn(move |req| {
                        metrics_handler(req, registry_clone.clone())
                    }),
                )
                .await
            {
                log::error!("Error serving connection: {e:?}");
            }
        });
    }
}
