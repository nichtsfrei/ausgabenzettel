#!/usr/bin/env bash

set -e

CA_NAME="MyPrivateCA"
DAYS_CA=3650
DAYS_CLIENT=365

PKI_DIR="./pki"
CA_DIR="$PKI_DIR/ca"
CLIENT_DIR="$PKI_DIR/clients"

mkdir -p "$CA_DIR" "$CLIENT_DIR"

CA_CONFIG="$CA_DIR/openssl.cnf"
CLIENT_CONFIG_TEMPLATE="$PKI_DIR/openssl-client.cnf.tpl"

# ----- Create CA config -----
cat > "$CA_CONFIG" <<EOF
[ ca ]
default_ca = CA_default

[ CA_default ]
dir               = $CA_DIR
database          = \$dir/index.txt
new_certs_dir     = \$dir/certs
certificate       = \$dir/$CA_NAME.crt
serial            = \$dir/serial
private_key       = \$dir/$CA_NAME.key
default_md        = sha256
default_days      = $DAYS_CA
policy            = policy_loose
email_in_dn       = no

[ policy_loose ]
commonName        = supplied

[ req ]
default_bits       = 4096
prompt             = no
default_md         = sha256
distinguished_name = dn

[ dn ]
CN = $CA_NAME

[ v3_ca ]
subjectKeyIdentifier=hash
authorityKeyIdentifier=keyid:always,issuer
basicConstraints = critical, CA:true
keyUsage = critical, digitalSignature, cRLSign, keyCertSign

[ req_ext ]
extendedKeyUsage = clientAuth
keyUsage = digitalSignature
EOF

mkdir -p "$CA_DIR/certs"
touch "$CA_DIR/index.txt"
SERIAL_FILE="$CA_DIR/serial"
[[ -f "$SERIAL_FILE" ]] && SERIAL=$(<"$SERIAL_FILE") || echo 1000 > "$SERIAL_FILE"
echo "SERIAL: $SERIAL"

cat > "$CLIENT_CONFIG_TEMPLATE" <<EOF
[ req ]
default_bits       = 2048
prompt             = no
default_md         = sha256
distinguished_name = dn
req_extensions     = req_ext

[ dn ]
CN = __CLIENT_NAME__

[ req_ext ]
extendedKeyUsage = clientAuth
keyUsage = digitalSignature
EOF


create_ca() {
    echo ">>> Creating CA private key..."
    openssl genpkey -algorithm RSA -out "$CA_DIR/$CA_NAME.key"

    echo ">>> Creating CA certificate..."
    openssl req -x509 -new -config "$CA_CONFIG" \
        -key "$CA_DIR/$CA_NAME.key" \
        -out "$CA_DIR/$CA_NAME.crt" \
        -days "$DAYS_CA" \
        -extensions v3_ca

    echo ">>> CA created at:"
    echo "     $CA_DIR/$CA_NAME.key"
    echo "     $CA_DIR/$CA_NAME.crt"
}

create_client() {
    CLIENT_NAME="$1"
    if [ -z "$CLIENT_NAME" ]; then
        echo "Usage: $0 client <name>"
        exit 1
    fi

    echo ">>> Generating client certificate: $CLIENT_NAME"

    CLIENT_WORK_DIR="$CLIENT_DIR/$CLIENT_NAME"
    mkdir -p "$CLIENT_WORK_DIR"

    CLIENT_CONFIG="$CLIENT_WORK_DIR/openssl.cnf"
    sed "s/__CLIENT_NAME__/$CLIENT_NAME/" "$CLIENT_CONFIG_TEMPLATE" > "$CLIENT_CONFIG"

    CLIENT_PARTIAL="$CLIENT_WORK_DIR/$CLIENT_NAME"

    openssl genpkey -algorithm RSA \
        -out "$CLIENT_PARTIAL.key"

    openssl req -new \
        -config "$CLIENT_CONFIG" \
        -key "$CLIENT_PARTIAL.key" \
        -out "$CLIENT_PARTIAL.csr"

    openssl ca -batch \
        -config "$CA_CONFIG" \
        -extensions req_ext \
        -in "$CLIENT_PARTIAL.csr" \
        -out "$CLIENT_PARTIAL.crt" \
        -days "$DAYS_CLIENT"

    openssl pkcs12 -export \
        -out "$CLIENT_PARTIAL.p12" \
        -inkey "$CLIENT_PARTIAL.key" \
        -in "$CLIENT_PARTIAL.crt"\
        -certfile "$CA_DIR/$CA_NAME.crt"


    echo ">>> Client certificate created:"
    echo "     $CLIENT_PARTIAL.key"
    echo "     $CLIENT_PARTIAL.crt"
    echo "     $CLIENT_PARTIAL.p12"
}

CMD="$1"

case "$CMD" in
    ca)
        create_ca
        ;;
    client)
        create_client "$2"
        ;;
    *)
        echo "Usage:"
        echo "  $0 ca                  # Create the CA"
        echo "  $0 client <name>       # Create a client certificate"
        exit 1
        ;;
esac
