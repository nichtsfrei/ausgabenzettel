# Ausgabenzettel

A personal expense tracking solution.

## Components

- **[CLI Tool (cli/)](./cli/README.md)**: `auseinnahmen` - Categorize banking CSV exports interactively
- **[Backend (backend/)](../backend/README.md)**: `ausgabenzettel` - HTTP API with mTLS authentication
- **[Frontend (frontend/)](../frontend/README.md)**: Web interface for expense tracking

## Quick Start

### CLI Tool

See [cli/README.md](cli/README.md) for details.

```bash
cd cli
cargo install --path .
```

### Backend & Frontend

```bash
cd frontend
make
make install

cd ../backend
cargo install --path .
```

The backend serves the frontend and requires mTLS authentication.

## Directory Structure

```
ausgabenzettel/
├── cli/              # CSV categorization CLI tool
├── backend/          # HTTP API server
└── frontend/         # Web application
```
