# Ambiente de Teste KeyHaven com Docker

Este diretório contém a infraestrutura Docker para testar o daemon e a CLI do KeyHaven em containers isolados.

## Arquitetura

```
┌──────────────────────────────────────────────────────────────┐
│                    Rede Docker                               │
│  ┌─────────────────────┐    ┌─────────────────────────────┐  │
│  │   keyhaven-daemon   │◄──►│        keyhaven-cli         │  │
│  │                     │    │                             │  │
│  │  - Mantém vault.db  │    │  - Comandos CLI             │  │
│  │  - Gerencia socket  │    │  - Conecta via socket       │  │
│  │  - Auto-bloqueio    │    │                             │  │
│  └─────────────────────┘    └─────────────────────────────┘  │
│           │                            │                     │
│           └────────────┬───────────────┘                     │
│                        │                                     │
│                 Volumes Compartilhados                       │
│         ┌──────────────┼──────────────┐                      │
│         ▼              ▼              ▼                      │
│    vault-data    vault-socket   vault-config                 │
│    (banco)        (comunicação)   (configurações)            │
└──────────────────────────────────────────────────────────────┘
```

## Início Rápido

### Construir e Iniciar

```bash
cd /home/Midir/Projetos/KeyHaven/test
docker-compose up --build -d
```

### Verificar Status

```bash
# Ver containers em execução
docker-compose ps

# Ver logs do daemon
docker-compose logs -f daemon

# Ver logs da CLI
docker-compose logs -f cli
```

### Usando a CLI

Acesse o container da CLI de forma interativa:

```bash
# Entrar no container da CLI
docker-compose exec cli bash

# Dentro do container, use o keyhaven:
/app/keyhaven --help

# Inicializar o cofre (se ainda não foi feito)
/app/keyhaven init

# Desbloquear o cofre
/app/keyhaven unlock

# Adicionar uma entrada
/app/keyhaven add github

# Listar entradas
/app/keyhaven list

# Obter uma senha
/app/keyhaven get github
```

### Parar

```bash
# Parar todos os serviços
docker-compose down

# Parar e remover volumes (ATENÇÃO: apaga os dados do cofre!)
docker-compose down -v
```

## Persistência de Volumes

| Volume | Propósito | Caminho no Container |
|--------|-----------|----------------------|
| `vault-data` | Banco de dados criptografado | `/data` |
| `vault-socket` | Socket Unix para comunicação daemon↔CLI | `/run/keyhaven` |
| `vault-config` | Arquivos de configuração | `/config` |

## Solução de Problemas

### Daemon não inicia

Verifique os logs:
```bash
docker-compose logs daemon
```

### CLI não conecta ao daemon

Verifique se o daemon está saudável:
```bash
docker-compose ps
```

Verifique se o socket existe:
```bash
docker-compose exec cli ls -la /run/keyhaven/
```

### Resetar tudo

```bash
docker-compose down -v
docker-compose up --build -d
```

## Fluxo de Desenvolvimento

Ao fazer alterações no código:

```bash
# Reconstruir após mudanças
docker-compose up --build -d

# Ou reconstruir serviço específico
docker-compose build daemon
docker-compose up -d daemon
```

## Usando o Script Auxiliar

Um script `keyhaven.sh` está disponível para facilitar:

```bash
# Construir e iniciar
./keyhaven.sh init

# Executar comandos CLI
./keyhaven.sh cli --help
./keyhaven.sh cli init
./keyhaven.sh cli unlock
./keyhaven.sh cli list

# Abrir shell interativo
./keyhaven.sh shell

# Ver status
./keyhaven.sh status

# Ver logs
./keyhaven.sh logs daemon

# Parar
./keyhaven.sh stop

# Resetar (apaga dados!)
./keyhaven.sh reset
```

## Usando o Makefile

Alternativamente, use o Make:

```bash
# Construir e iniciar
make init

# Executar comandos CLI
make cli cmd=--help
make cli cmd=init

# Abrir shell
make shell

# Parar
make stop
```
