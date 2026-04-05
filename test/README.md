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
cd /home/Midir/Projetos/KeyHaven/test
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

| Volume | Purpose | Path in Container |
|--------|---------|-------------------|
| `vault-data` | Encrypted vault database | `/data` |
| `vault-socket` | Unix socket for daemon↔CLI communication | `/run/keyhaven` |
| `vault-config` | Configuration files | `/config` |

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

## Development Workflow

When making changes to the code:

```bash
# Rebuild after code changes
docker-compose up --build -d

# Or rebuild specific service
docker-compose build daemon
docker-compose up -d daemon
```
