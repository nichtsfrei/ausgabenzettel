use rustls::server::WebPkiClientVerifier;
// use crate::{
//     Address,
//     accept::{Accept, DefaultAcceptor},
//     server::{Server, io_other},
// };
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{RootCertStore, ServerConfig};
use std::{io, path::Path};

pub fn io_other<E>(error: E) -> io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    io::Error::other(error)
}

/// Create config from PEM formatted files.
///
/// Contents of certificate file and private key file must be in PEM format.
pub async fn from_pem_file(
    cert: impl AsRef<Path>,
    key: impl AsRef<Path>,
    ca: impl AsRef<Path>,
) -> io::Result<ServerConfig> {
    let cert = fs_err::tokio::read(cert.as_ref()).await?;
    let key = fs_err::tokio::read(key.as_ref()).await?;
    let ca = fs_err::tokio::read(ca.as_ref()).await?;

    config_from_pem(cert, key, ca)
}

fn config_from_der(
    cert: Vec<Vec<u8>>,
    key: Vec<u8>,
    client_ca: Vec<Vec<u8>>,
) -> io::Result<ServerConfig> {
    let cert = cert.into_iter().map(CertificateDer::from).collect();
    let key = PrivateKeyDer::try_from(key).map_err(io_other)?;
    let cas = client_ca
        .into_iter()
        .map(CertificateDer::from)
        .collect::<Vec<_>>();
    let mut root_ca = RootCertStore::empty();
    for ca in cas {
        root_ca.add(ca).map_err(io_other)?;
    }
    let client_cert_verifier = WebPkiClientVerifier::builder(root_ca.into())
        .build()
        .map_err(io_other)?;

    let mut config = ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(cert, key)
        .map_err(io_other)?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(config)
}

fn config_from_pem(cert: Vec<u8>, key: Vec<u8>, ca: Vec<u8>) -> io::Result<ServerConfig> {
    let cert: Vec<CertificateDer> = CertificateDer::pem_slice_iter(&cert)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| io_other("failed to parse certificate"))?;
    let client_ca: Vec<CertificateDer> = CertificateDer::pem_slice_iter(&ca)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| io_other("failed to parse client_ca"))?;

    let mut key_result: Result<PrivateKeyDer, io::Error> =
        Err(io_other("The private key file contained no keys"));

    for item in rustls::pki_types::pem::PemObject::pem_slice_iter(&key) {
        let key: Result<PrivateKeyDer, io::Error> =
            item.map_err(|_| io_other("failed to parse PEM"));

        match key_result {
            Ok(_) => {
                if key.is_ok() {
                    return Err(io_other(
                        "The private key file containsed multiple keys (it must only contain one)",
                    ));
                }
            }
            Err(_) => key_result = key,
        }
    }

    let key = key_result?;
    let cert_der: Vec<Vec<u8>> = cert.into_iter().map(|c| c.to_vec()).collect();
    let client_ca_cer: Vec<Vec<u8>> = client_ca.into_iter().map(|c| c.to_vec()).collect();
    let key_der = key.secret_der().to_vec();

    config_from_der(cert_der, key_der, client_ca_cer)
}
