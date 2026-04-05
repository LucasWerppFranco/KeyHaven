# KeyHaven Docker Test Environment Makefile

.PHONY: help init start stop restart status logs shell cli reset clean

help:
	@echo "KeyHaven Docker Test Environment"
	@echo ""
	@echo "Available targets:"
	@echo "  init      - Build and start containers"
	@echo "  start     - Start containers"
	@echo "  stop      - Stop containers"
	@echo "  restart   - Restart containers"
	@echo "  status    - Show container status"
	@echo "  logs      - View logs (use 'make logs svc=daemon')"
	@echo "  shell     - Open shell in CLI container"
	@echo "  cli       - Run CLI command (use 'make cli cmd=--help')"
	@echo "  reset     - Delete all data (WARNING!)"
	@echo "  clean     - Remove everything (WARNING!)"

init:
	@echo "Building and starting KeyHaven..."
	docker-compose up --build -d
	@echo "Done! Run 'make cli cmd=--help' to get started."

start:
	docker-compose up -d

stop:
	docker-compose stop

restart:
	docker-compose restart

status:
	docker-compose ps

logs:
ifdef svc
	docker-compose logs -f $(svc)
else
	docker-compose logs -f
endif

shell:
	docker-compose exec cli bash

cli:
ifdef cmd
	docker-compose exec cli /app/keyhaven $(cmd)
else
	docker-compose exec cli /app/keyhaven
endif

reset:
	@echo "WARNING: This will delete all vault data!"
	@read -p "Are you sure? (yes/no): " confirm; \
	if [ "$$confirm" = "yes" ]; then \
		docker-compose down -v; \
		echo "All data has been deleted."; \
	else \
		echo "Cancelled."; \
	fi

clean:
	@echo "WARNING: This will remove all containers, volumes, and images!"
	@read -p "Are you sure? (yes/no): " confirm; \
	if [ "$$confirm" = "yes" ]; then \
		docker-compose down -v --rmi all; \
		echo "Cleanup complete."; \
	else \
		echo "Cancelled."; \
	fi
