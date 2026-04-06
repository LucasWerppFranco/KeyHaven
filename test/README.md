# KeyHaven Docker Test Environment

This directory contains Docker infrastructure for testing the KeyHaven daemon and CLI in isolated containers.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Docker Network                            │
│  ┌─────────────────────┐    ┌─────────────────────────────┐  │
│  │   keyhaven-daemon   │◄──►│        keyhaven-cli         │  │
│  │                     │    │                             │  │
│  │  - Holds vault.db   │    │  - CLI commands             │  │
│  │  - Manages socket   │    │  - Connects via socket      │  │
│  │  - Auto-locks       │    │                             │  │
│  └─────────────────────┘    └─────────────────────────────┘  │
│           │                            │                     │
│           └────────────┬───────────────┘                     │
│                        │                                     │
│                 Shared Volumes                               │
│         ┌──────────────┼──────────────┐                      │
│         ▼              ▼              ▼                      │
│    vault-data    vault-socket   vault-config                 │
│    (database)    (IPC comms)    (settings)                   │
└──────────────────────────────────────────────────────────────┘
```

## Quick Start

### Build and Start

```bash
cd test
docker-compose up --build -d
```

### Check Status

```bash
# See running containers
docker-compose ps

# View daemon logs
docker-compose logs -f daemon

# View CLI logs
docker-compose logs -f cli
```

### Using the CLI

Access the CLI container interactively:

```bash
# Enter the CLI container
docker-compose exec cli bash

# Now inside the container, use keyhaven:
/app/keyhaven --help

# Initialize vault (if not already done)
/app/keyhaven init

# Unlock vault
/app/keyhaven unlock

# Add an entry
/app/keyhaven add github

# List entries
/app/keyhaven list

# Get a password
/app/keyhaven get github
```

### Stopping

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (WARNING: deletes vault data!)
docker-compose down -v
```

## Volume Persistence

| Volume       | Purpose                                  | Path in Container |
|--------------|------------------------------------------|-------------------|
| `vault-data`   | Encrypted vault database                 | `/data`             |
| `vault-socket` | Unix socket for daemon↔CLI communication | `/run/keyhaven`     |
| `vault-config` | Configuration files                      | `/config`           |

## Troubleshooting

### Daemon not starting

Check logs:
```bash
docker-compose logs daemon
```

### CLI can't connect to daemon

Ensure daemon is healthy:
```bash
docker-compose ps
```

Check if socket exists:
```bash
docker-compose exec cli ls -la /run/keyhaven/
```

### Reset everything

```bash
docker-compose down -v
docker-compose up --build -d
```

## Using the Helper Script

A `keyhaven.sh` script is available for convenience:

```bash
# Build and start
./keyhaven.sh init

# Run CLI commands
./keyhaven.sh cli --help
./keyhaven.sh cli init
./keyhaven.sh cli unlock
./keyhaven.sh cli list

# Open interactive shell
./keyhaven.sh shell

# Check status
./keyhaven.sh status

# View logs
./keyhaven.sh logs daemon

# Stop
./keyhaven.sh stop

# Reset (deletes data!)
./keyhaven.sh reset
```

## Using the Makefile

Alternatively, use Make:

```bash
# Build and start
make init

# Run CLI commands
make cli cmd=--help
make cli cmd=init

# Open shell
make shell

# Stop
make stop
```

## Alternative: Native Testing

If Docker builds fail due to network timeouts, you can run tests natively:

```bash
# Run tests on core crates (excludes desktop GUI)
cargo test -p vault-core -p vault-daemon -p vault-cli
```

## Development Workflow

When making changes to the code:

```bash
# Rebuild after code changes
docker-compose up --build -d

# Or rebuild specific service
docker-compose build daemon
docker-compose up -d daemon
```
