#!/bin/bash
# Helper script to run KeyHaven CLI commands in the Docker container

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

show_help() {
    cat << EOF
KeyHaven Docker Helper Script

Usage: $0 [command] [options]

Commands:
    init          Initialize the Docker environment (build and start)
    start         Start the daemon and CLI containers
    stop          Stop all containers
    restart       Restart all containers
    status        Show container status
    logs [svc]    View logs (daemon|cli)
    cli [args]    Run KeyHaven CLI command (e.g., ./keyhaven.sh cli --help)
    shell         Open a shell in the CLI container
    reset         Stop and remove all data (WARNING: deletes vault!)
    clean         Remove all containers, volumes, and images

Examples:
    $0 init
    $0 cli init
    $0 cli unlock
    $0 cli list
    $0 logs daemon
EOF
}

check_docker() {
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}Error: Docker is not installed${NC}"
        exit 1
    fi
    if ! command -v docker-compose &> /dev/null; then
        echo -e "${RED}Error: docker-compose is not installed${NC}"
        exit 1
    fi
}

init() {
    echo -e "${GREEN}Building and starting KeyHaven Docker environment...${NC}"
    cd "$SCRIPT_DIR"
    docker-compose up --build -d
    echo -e "${GREEN}Done! Use '$0 cli --help' to get started.${NC}"
}

start() {
    echo -e "${GREEN}Starting KeyHaven containers...${NC}"
    cd "$SCRIPT_DIR"
    docker-compose up -d
}

stop() {
    echo -e "${YELLOW}Stopping KeyHaven containers...${NC}"
    cd "$SCRIPT_DIR"
    docker-compose stop
}

restart() {
    echo -e "${YELLOW}Restarting KeyHaven containers...${NC}"
    cd "$SCRIPT_DIR"
    docker-compose restart
}

status() {
    cd "$SCRIPT_DIR"
    docker-compose ps
}

logs() {
    cd "$SCRIPT_DIR"
    if [ -z "$1" ]; then
        docker-compose logs -f
    else
        docker-compose logs -f "$1"
    fi
}

cli() {
    cd "$SCRIPT_DIR"
    # Check if daemon is running
    if ! docker-compose ps | grep -q "Up"; then
        echo -e "${YELLOW}Warning: Containers are not running. Starting them...${NC}"
        docker-compose up -d
        sleep 2
    fi
    # Execute CLI command
    docker-compose exec cli /app/keyhaven "$@"
}

shell() {
    cd "$SCRIPT_DIR"
    echo -e "${GREEN}Opening shell in CLI container...${NC}"
    echo -e "${YELLOW}Tip: Run '/app/keyhaven --help' to see available commands${NC}"
    docker-compose exec cli bash
}

reset() {
    echo -e "${RED}WARNING: This will delete all vault data!${NC}"
    read -p "Are you sure? (yes/no): " confirm
    if [ "$confirm" = "yes" ]; then
        cd "$SCRIPT_DIR"
        docker-compose down -v
        echo -e "${GREEN}All data has been deleted.${NC}"
    else
        echo "Cancelled."
    fi
}

clean() {
    echo -e "${RED}WARNING: This will remove all containers, volumes, and images!${NC}"
    read -p "Are you sure? (yes/no): " confirm
    if [ "$confirm" = "yes" ]; then
        cd "$SCRIPT_DIR"
        docker-compose down -v --rmi all
        echo -e "${GREEN}Cleanup complete.${NC}"
    else
        echo "Cancelled."
    fi
}

# Main
main() {
    check_docker

    case "${1:-help}" in
        init)
            init
            ;;
        start)
            start
            ;;
        stop)
            stop
            ;;
        restart)
            restart
            ;;
        status)
            status
            ;;
        logs)
            shift
            logs "$@"
            ;;
        cli)
            shift
            cli "$@"
            ;;
        shell)
            shell
            ;;
        reset)
            reset
            ;;
        clean)
            clean
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo -e "${RED}Unknown command: $1${NC}"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
