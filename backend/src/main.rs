//! Run wit
//!
//! ```not_rust
//! cargo run -p example-tls-rustls
//! ```
mod certs;

use axum::{
    BoxError, Router,
    body::{Body, Bytes},
    extract::Request,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, head, put},
};
use axum_server::tls_rustls::RustlsConfig;
use futures_util::Stream;
use futures_util::TryStreamExt;
use ring::digest::{Context, SHA256};
use std::{
    io::{self},
    net::SocketAddr,
    path::{Path, PathBuf},
    pin::pin,
};
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::{ReaderStream, StreamReader};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cert_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("certs");

    // configure certificate and private key used by https
    let config = certs::from_pem_file(
        cert_dir.join("cert.pem"),
        cert_dir.join("key.pem"),
        cert_dir.join("ca.crt"),
    )
    .await
    .unwrap();

    let app = Router::new()
        .route("/", put(save))
        .route("/", get(get_html))
        .route("/", head(header));
    let config = RustlsConfig::from_config(config.into());

    // run https server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn header() -> impl IntoResponse {
    let path = path("current.html");
    let etag = etag_of(path).await;
    let header = [(header::ETAG, etag)];
    (header, StatusCode::OK)
}

async fn save(request: Request) -> Result<impl IntoResponse, StatusCode> {
    match request.headers().get(header::IF_MATCH) {
        None => {
            tracing::warn!("if match header missing");
            Ok(stream_file(StatusCode::NOT_ACCEPTABLE, "current.html").await)
        }
        // TODO: error handling
        Some(etag) => {
            let etag = etag.to_str().unwrap();
            let path = path("current.html");
            let current_etag = etag_of(path).await;
            if etag != current_etag {
                tracing::warn!(etag, current_etag, "Wrong etag");
                Ok(stream_file(StatusCode::NOT_ACCEPTABLE, "current.html").await)
            } else {
                stream_to_file("current.html", request.into_body().into_data_stream()).await?;
                Ok(stream_file(StatusCode::OK, "current.html").await)
            }
        }
    }
}

fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek()
        && !matches!(first, std::path::Component::Normal(_))
    {
        return false;
    }

    components.count() == 1
}
async fn store<S, E>(name: &str, stream: S) -> Result<(), io::Error>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    let body_with_io_error = stream.map_err(io::Error::other);
    let mut body_reader = pin!(StreamReader::new(body_with_io_error));

    // TODO: use git to commit and add new versions
    let path = std::path::Path::new("upload").join(name);
    let mut file = BufWriter::new(File::create(&path).await?);
    tracing::debug!(?path, "copying body");
    tokio::io::copy(&mut body_reader, &mut file).await?;
    tracing::debug!(?path, "copyied body");
    // TODO: calculate sha256sum and return it as etag from body instead of from file
    let etag = sha256_digest(path).await;
    let sha256path = std::path::Path::new("upload").join(format!("{name}.sha256sum"));
    tracing::debug!(?sha256path, %etag, "etag");
    let mut file = BufWriter::new(File::create(&sha256path).await?);
    tokio::io::copy(&mut etag.as_bytes(), &mut file).await?;
    tracing::debug!(?sha256path, %etag, "stored");

    Ok(())
}

async fn stream_to_file<S, E>(name: &str, stream: S) -> Result<(), StatusCode>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(name) {
        tracing::info!(name, "invalid");
        return Err(StatusCode::BAD_REQUEST);
    }

    store(name, stream).await.map_err(|error| {
        tracing::error!(%error, "Unable to store content");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn sha256_digest(path: PathBuf) -> String {
    tokio::task::spawn_blocking(move || {
        use std::fs::File;
        use std::io::{BufReader, Read};
        let mut context = Context::new(&SHA256);
        let mut buffer = [0; 1024];
        tracing::debug!(?path, "Calculating hash");
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);

        loop {
            let count = reader.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            context.update(&buffer[..count]);
        }

        data_encoding::HEXUPPER.encode(context.finish().as_ref())
    })
    .await
    .unwrap()
}

async fn etag_of<P>(p: P) -> String
where
    P: AsRef<Path>,
{
    let hash_file = p.as_ref().join(".sha256");
    if hash_file.is_file() {
        match tokio::fs::read_to_string(&hash_file).await {
            Ok(x) => x,
            Err(error) => {
                tracing::warn!(%error, ?hash_file, "Unable to read, returning empty.");
                Default::default()
            }
        }
    } else {
        sha256_digest(p.as_ref().into()).await
    }
}

fn path(file: &str) -> PathBuf {
    let current = std::path::Path::new("upload").join(file);
    if current.exists() && current.is_file() {
        current
    } else {
        tracing::debug!(file, "not found. Returning initial");
        std::path::Path::new("initial").join("index.html")
    }
}

async fn stream_file(status: StatusCode, file: &str) -> impl IntoResponse {
    let path = path(file);
    let etag = etag_of(&path).await;
    let header = [
        (header::CONTENT_TYPE, "text/html".to_string()),
        (header::ETAG, etag),
    ];

    let file = File::open(path).await.unwrap(); // previously checked
    let stream = ReaderStream::new(file);
    (status, header, Body::from_stream(stream))
}

async fn get_html() -> impl IntoResponse {
    stream_file(StatusCode::OK, "current.html").await
}
