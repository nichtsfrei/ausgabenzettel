mod certs;
mod config;

use axum::{
    BoxError, Router,
    body::{Body, Bytes},
    extract::{Request, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, head, put},
};
use futures_util::TryStreamExt;
use futures_util::{Stream, StreamExt};
use ring::digest::{Context, SHA256};
use std::{
    io::{self},
    path::{Path, PathBuf},
    pin::pin,
};
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::{ReaderStream, StreamReader};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

enum ReturnType {
    Full,
    Content,
}

#[derive(Clone)]
struct PageStreamer {
    upload: PathBuf,
    empty_content_etag: String,
}

impl PageStreamer {
    const HEADER: &'static [u8] = include_bytes!("../initial/head.template");
    const EMPTY_CONTENT: &'static [u8] = include_bytes!("../initial/content.template");
    const TAIL: &'static [u8] = include_bytes!("../initial/tail.template");
    fn path(&self, file: &str) -> Option<PathBuf> {
        let current = self.upload.join(file);
        if current.exists() && current.is_file() {
            Some(current)
        } else {
            None
        }
    }

    async fn etag(&self, path: Option<&PathBuf>) -> String {
        if let Some(path) = path.as_ref() {
            etag_of(path).await
        } else {
            self.empty_content_etag.clone()
        }
    }

    async fn stream_file(
        self,
        rt: ReturnType,
        status: StatusCode,
        content_path: &str,
    ) -> impl IntoResponse {
        let path = self.path(content_path);
        let header = [
            (header::CONTENT_TYPE, "text/html".to_string()),
            (header::ETAG, self.etag(path.as_ref()).await),
        ];
        match rt {
            ReturnType::Full => {
                // head.template, content, tail.template
                let head = ReaderStream::new(Self::HEADER);
                let tail = ReaderStream::new(Self::TAIL);
                if let Some(path) = path.as_ref() {
                    let file = File::open(path).await.unwrap();
                    let content = ReaderStream::new(file);
                    let stream = head.chain(content).chain(tail);
                    (status, header, Body::from_stream(stream))
                } else {
                    let content = ReaderStream::new(Self::EMPTY_CONTENT);
                    let stream = head.chain(content).chain(tail);
                    (status, header, Body::from_stream(stream))
                }
            }
            ReturnType::Content => {
                if let Some(path) = path.as_ref() {
                    let file = File::open(path).await.unwrap();
                    let content = ReaderStream::new(file);
                    (status, header, Body::from_stream(content))
                } else {
                    let content = ReaderStream::new(Self::EMPTY_CONTENT);
                    (status, header, Body::from_stream(content))
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::init().await?;
    let ps = PageStreamer {
        upload: config.upload_dir,
        empty_content_etag: "INITIAL".into(),
    };

    let app = Router::new()
        .route("/", put(save))
        .route("/", get(get_html))
        .route("/", head(header))
        .with_state(ps);

    // run https server
    tracing::info!("listening on {}", config.listening);
    axum_server::bind_rustls(config.listening, config.tls)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn header(State(ps): State<PageStreamer>) -> impl IntoResponse {
    let path = ps.path("current.html");
    let etag = ps.etag(path.as_ref()).await;
    let header = [(header::ETAG, etag)];
    (header, StatusCode::OK)
}

async fn save(
    State(ps): State<PageStreamer>,
    request: Request,
) -> Result<impl IntoResponse, StatusCode> {
    let sc = match request.headers().get(header::IF_MATCH) {
        None => {
            tracing::warn!("if match header missing");
            StatusCode::NOT_ACCEPTABLE
        }
        Some(etag) => {
            let etag = etag.to_str().unwrap();
            let current_etag = ps.etag(ps.path("current.html").as_ref()).await;
            if etag != current_etag {
                tracing::warn!(etag, current_etag, "Wrong etag");
                StatusCode::CONFLICT
            } else {
                stream_to_file(
                    &ps.upload,
                    "current.html",
                    request.into_body().into_data_stream(),
                )
                .await?;
                StatusCode::OK
            }
        }
    };

    Ok(ps
        .stream_file(ReturnType::Content, sc, "current.html")
        .await)
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
async fn store<S, E>(base: &Path, name: &str, stream: S) -> Result<(), io::Error>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    let body_with_io_error = stream.map_err(io::Error::other);
    let mut body_reader = pin!(StreamReader::new(body_with_io_error));
    let path = base.join(name);
    tracing::debug!(?path, "file storage");
    let mut file = BufWriter::new(File::create(&path).await?);
    tokio::io::copy(&mut body_reader, &mut file).await?;
    // TODO: calculate sha256sum and return it as etag from body instead of from file
    let etag = sha256_digest(path).await;
    let sha256path = base.join(format!("{name}.sha256sum"));
    tracing::debug!(?sha256path, %etag, "etag");
    let mut file = BufWriter::new(File::create(&sha256path).await?);
    tokio::io::copy(&mut etag.as_bytes(), &mut file).await?;
    tracing::debug!(?sha256path, %etag, "stored");

    Ok(())
}

async fn stream_to_file<S, E>(base: &Path, name: &str, stream: S) -> Result<(), StatusCode>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(name) {
        tracing::info!(name, "invalid");
        return Err(StatusCode::BAD_REQUEST);
    }

    store(base, name, stream).await.map_err(|error| {
        tracing::error!(name, %error, "Unable to store content");
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

async fn get_html(State(ps): State<PageStreamer>) -> impl IntoResponse {
    ps.stream_file(ReturnType::Full, StatusCode::OK, "current.html")
        .await
}
