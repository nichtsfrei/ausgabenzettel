# Ausgabenzettel

Is a very simple and opinionated solution to track personal expenses.

## Frontend Generation

If you want to just use the frontend with local storage and only file export functionality for quick testing use `make` within `frontend`.

```
cd frontend
make
```

That generates a `index.html` that can be opened in any browser supporting JavaScript and CSS.

To enroll it into the backend:
```
make install
```

To change categories please use the `labels` constant within `root.js`.

## Backend Installation

The backend uses
- `header.template`,
- `body.html`,
- `tail.template`

Within its `template` directory to embed them into the binary.

Therefore, `make install` must be executed in `../frontend` whenever the
frontend changes and should be served by the backend.

```
cd frontend && make install
cd backend && cargo install --path .
```

The backend service enforces `mTLS` and requires:

- `server.cer`,
- `server.key`,
- `ca.cer`,

Within either `$XDG_CONFIG_HOME/ausgabenzettel`, `$HOME/.config/ausgabenzettel`
or `/etc/ausgabenzettel`.


To generate the `ca.cer` as well as client keys and certificates you can use 

```bash
# Generates ./pki/ca/MyPrivateCA.crt that can copied to ca.cer
bash gen-client-certs.bash ca
bash gen-client-certs.bash client testuser
```

All clients must present a certificate signed by the configured CA.

As it is storing the user data directly onto to the filesystem it requires
write access to either:

- `XDG_RUNTIME_DIR/ausgabenzettel`
- `/var/lib/ausgabenzettel`
 

To set the listening address use the environment variable
`AUSGABENZETTEL_LISTENING` e.g.:

```bash
AUSGABENZETTEL_LISTENING=0.0.0.0:3000 ausgabenzettel
```

