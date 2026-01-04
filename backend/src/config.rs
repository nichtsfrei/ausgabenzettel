use std::{
    env,
    net::{AddrParseError, SocketAddr},
    path::{Path, PathBuf},
};

use axum_server::tls_rustls::RustlsConfig;

use crate::certs;

pub struct Certificates {
    server_cert: PathBuf,
    server_key: PathBuf,
    client_ca: PathBuf,
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum CertificateKind {
    ServerCert,
    ServerKey,
    ClientCa,
}

impl std::fmt::Display for CertificateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for CertificateKind {
    fn as_ref(&self) -> &str {
        match self {
            CertificateKind::ServerCert => "server.cer",
            CertificateKind::ServerKey => "server.key",
            CertificateKind::ClientCa => "ca.cer",
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Kind {
    #[error(transparent)]
    Certificate(#[from] CertificateKind),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0} not found")]
    NotFound(Kind),
    #[error("{0} is not a dir")]
    InvalidDataDir(PathBuf),
    #[error("{0} unable to create")]
    UnableToCreateDataDir(#[from] std::io::Error),
    #[error(transparent)]
    InvalidAddres(#[from] AddrParseError),
}

const APPLICATION_NAME: &str = "ausgabenzettel";

impl Certificates {
    fn check_for(user: &Path, system: &Path, kind: CertificateKind) -> Result<PathBuf, Error> {
        let user = user.join(kind.as_ref());
        if user.exists() {
            tracing::debug!(%kind, ?user);
            return Ok(user);
        }
        let system = system.join(kind.as_ref());
        if system.exists() {
            tracing::debug!(%kind, ?system);
            return Ok(system);
        }
        tracing::warn!(?user, ?system, %kind, "not found");
        Err(Error::NotFound(Kind::Certificate(kind)))
    }

    pub fn init() -> Result<Certificates, Error> {
        let user = env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                env::var("HOME")
                    .map(PathBuf::from)
                    .map(|x| x.join(".config"))
            })
            .unwrap_or_else(|_| PathBuf::from("~/.config"))
            .join(APPLICATION_NAME);

        let system = PathBuf::from("/etc").join(APPLICATION_NAME);
        let server_cert = Self::check_for(&user, &system, CertificateKind::ServerCert)?;
        let server_key = Self::check_for(&user, &system, CertificateKind::ServerKey)?;
        let client_ca = Self::check_for(&user, &system, CertificateKind::ClientCa)?;
        Ok(Certificates {
            server_cert,
            server_key,
            client_ca,
        })
    }

    async fn into_rustls_config(self) -> Result<RustlsConfig, Error> {
        let cert_config =
            certs::from_pem_file(&self.server_cert, &self.server_key, &self.client_ca).await?;
        Ok(RustlsConfig::from_config(cert_config.into()))
    }
}

pub struct BasePaths {
    certificates: Certificates,
    client_data: PathBuf,
}

impl BasePaths {
    pub fn init() -> Result<Self, Error> {
        let certificates = Certificates::init()?;
        let user_data_path = env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/lib"))
            .join(APPLICATION_NAME);
        if !user_data_path.is_dir() {
            if user_data_path.exists() {
                return Err(Error::InvalidDataDir(user_data_path));
            }
            std::fs::create_dir(&user_data_path)?;
        }

        tracing::info!(?user_data_path, "Storing into");

        Ok(BasePaths {
            certificates,
            client_data: user_data_path,
        })
    }
}

pub struct Config {
    pub tls: RustlsConfig,
    pub listening: SocketAddr,
    pub upload_dir: PathBuf,
}

impl Config {
    pub async fn init() -> Result<Self, Error> {
        let paths = BasePaths::init()?;
        let listening = env::var("AUSGABENZETTEL_LISTENING")
            .unwrap_or("127.0.0.1:3000".into())
            .parse()?;
        let tls = paths.certificates.into_rustls_config().await?;
        let upload_dir = paths.client_data;

        Ok(Self {
            listening,
            tls,
            upload_dir,
        })
    }
}
