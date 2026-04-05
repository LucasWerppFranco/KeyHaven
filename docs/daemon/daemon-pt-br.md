# Daemon do KeyHaven

O **vault-daemon** é o serviço central que gerencia o cofre criptografado e serve como ponto único de acesso para todas as operações dos clientes. Ele executa como um processo em segundo plano e se comunica via sockets Unix.

## Índice

- [Visão Geral da Arquitetura](#visão-geral-da-arquitetura)
- [Modelo de Segurança](#modelo-de-segurança)
- [Protocolo de Comunicação](#protocolo-de-comunicação)
- [Gerenciamento de Sessão](#gerenciamento-de-sessão)
- [Configuração](#configuração)
- [Referência da API](#referência-da-api)
- [Executando o Daemon](#executando-o-daemon)

---

## Visão Geral da Arquitetura

```
┌─────────────────────────────────────────┐
│                Clientes                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │   CLI   │  │   GUI   │  │Navegador│  │
│  └────┬────┘  └────┬────┘  └────┬────┘  │
└───────┼────────────┼────────────┼───────┘
        │            │            │
        └────────────┴────────────┘
                     │
              Socket de Domínio Unix
                     │
        ┌────────────▼──────────┐
        │     vault-daemon      │
        │  ┌─────────────────┐  │
        │  │  Estado da      │  │
        │  │  Sessão         │  │
        │  │  (Chave Deriva- │  │
        │  │   da)           │  │
        │  └─────────────────┘  │
        │  ┌─────────────────┐  │
        │  │  Roteador de    │  │
        │  │  Requisições    │  │
        │  └─────────────────┘  │
        └────────────┬──────────┘
                     │
              SQLite + AES-256-GCM
                     │
              ┌──────▼──────┐
              │  vault.db   │
              └─────────────┘
```

O daemon segue uma arquitetura **processo único, multi-cliente**:

- Uma instância do daemon gerencia um banco de dados do cofre
- Múltiplos clientes podem conectar simultaneamente via socket Unix
- O estado da sessão é compartilhado entre todas as conexões (bloqueio/desbloqueio global)

---

## Modelo de Segurança

### Permissões do Socket Unix

O daemon cria um socket Unix com **permissões 0600** (somente leitura/escrita do proprietário). Isso garante:

- Somente o usuário proprietário pode conectar ao daemon
- Outros usuários no sistema não podem acessar o cofre

### Estado da Sessão

Quando o cofre é desbloqueado:

1. A chave de criptografia derivada é mantida na memória, envolvida em `Zeroizing<Vec<u8>>`
2. `Zeroizing` garante que a memória seja apagada com segurança quando a sessão termina
3. A chave nunca sai do processo do daemon

### Bloqueio Automático

O daemon bloqueia automaticamente o cofre após um período de inatividade:

- Tempo limite padrão: **15 minutos**
- Intervalo de verificação: **30 segundos**
- Configurável via `config.toml`

Quando bloqueado automaticamente, a chave é descartada e a memória é zerada.

---

## Protocolo de Comunicação

O daemon usa um protocolo simples baseado em mensagens sobre sockets Unix.

### Formato da Mensagem

Cada mensagem tem um prefixo de tamanho:

```
┌───────────┬─────────────────────────────────────┐
│  Tamanho  │              Payload                │
│ (4 bytes) │          (JSON, variável)           │
│  big-u32  │                                     │
└───────────┴─────────────────────────────────────┘
```

### Estrutura da Requisição

```json
{
  "id": "id-único-da-requisição",
  "action": "Unlock",
  "params": {}
}
```

**Ações:**

| Categoria | Ações                                                     |
|-----------|-----------------------------------------------------------|
| Sessão    | `Unlock`, `Lock`, `Status`                                      |
| Entradas  | `ListEntries`, `GetEntry`, `AddEntry`, `UpdateEntry`, `DeleteEntry` |
| Gerador   | `GeneratePassword`, `CheckPassword`                           |

### Estrutura da Resposta

```json
{
  "id": "mesmo-id-da-requisição",
  "ok": true,
  "data": { ... },
  "error": null
}
```

Ou em caso de erro:

```json
{
  "id": "mesmo-id-da-requisição",
  "ok": false,
  "data": null,
  "error": "Cofre bloqueado. Execute: vault unlock"
}
```

---

## Gerenciamento de Sessão

### Fluxo de Desbloqueio

```
Cliente                   Daemon                    vault-core
  │                       │                            │
  │────── unlock ───────> │                            │
  │    {senha}            │                            │
  │                       │─── derive_key() ───────>   │
  │                       │    {salt, parâmetros       │
  │                       │     Argon2id}              │
  │                       │ <──── chave derivada ───── │
  │                       │                            │
  │                       │  Armazena chave na Sessão  │
  │                       │  (Zeroizing<Vec<u8>>)      │
  │                       │                            │
  │ <──── sucesso ────────│                            │
  │    {timeout_secs}     │                            │
```

### Derivação de Chave

Ao desbloquear, o daemon:

1. Lê os parâmetros Argon2id e o salt da tabela `vault_meta`
2. Deriva a chave AES-256 usando Argon2id com os parâmetros armazenados
3. Verifica o HMAC-SHA256 de "vault-v1-ok" contra a tag de verificação armazenada
4. Se a verificação passar, armazena a chave no estado da sessão

Este design permite futuras atualizações de parâmetros sem invalidar cofres antigos.

### Máquina de Estados da Sessão

```
┌─────────────┐
│  Bloqueado  │<─────────────────────────┐
│  (sem       │                          │
│   chave)    │                          │
└──────┬──────┘                          │
       │ unlock                          │
       │ (verifica HMAC)                 │
       ▼                                 │
┌─────────────┐      tempo limite de     │
│ Desbloqueado│ ──── inatividade ou      │
│ (chave na   │      bloqueio explícito  │
│  memória)   │──────────────────────────┤
└─────────────┘──────────────────────────┘
```

---

## Configuração

O daemon carrega a configuração de `~/.config/vault/config.toml`.

### Configuração Padrão

```toml
# Caminho para o banco de dados SQLite criptografado
db_path = "~/.local/share/vault/vault.db"

# Caminho para o socket Unix
socket_path = "/tmp/vault.sock"  # ou $XDG_RUNTIME_DIR/vault.sock

# Tempo de inatividade antes do bloqueio automático (em segundos)
session_timeout = 900  # 15 minutos
```

### Precedência da Configuração

1. Arquivo de configuração do usuário (`~/.config/vault/config.toml`)
2. Padrões embutidos

---

## Referência da API

### Ações de Sessão

#### `Unlock`

Desbloqueia o cofre com a senha mestre.

**Requisição:**
```json
{
  "id": "req-1",
  "action": "Unlock",
  "params": {
    "password": "minha-senha-mestre"
  }
}
```

**Resposta (sucesso):**
```json
{
  "id": "req-1",
  "ok": true,
  "data": {
    "message": "Cofre desbloqueado",
    "timeout_secs": 900
  }
}
```

**Resposta (erro):**
```json
{
  "id": "req-1",
  "ok": false,
  "error": "Senha mestre incorreta"
}
```

---

#### `Lock`

Bloqueia imediatamente o cofre e apaga a chave da memória.

**Requisição:**
```json
{
  "id": "req-2",
  "action": "Lock",
  "params": {}
}
```

**Resposta:**
```json
{
  "id": "req-2",
  "ok": true,
  "data": {
    "message": "Cofre bloqueado"
  }
}
```

---

#### `Status`

Retorna se o cofre está atualmente desbloqueado.

**Requisição:**
```json
{
  "id": "req-3",
  "action": "Status",
  "params": {}
}
```

**Resposta:**
```json
{
  "id": "req-3",
  "ok": true,
  "data": {
    "unlocked": true
  }
}
```

---

### Ações de Entrada

Todas as ações de entrada requerem que o cofre esteja desbloqueado. Retorna erro se estiver bloqueado.

#### `ListEntries`

Lista as entradas do cofre com filtro de busca opcional.

**Requisição:**
```json
{
  "id": "req-4",
  "action": "ListEntries",
  "params": {
    "search": "github"
  }
}
```

**Resposta:**
```json
{
  "id": "req-4",
  "ok": true,
  "data": [
    {
      "id": 1,
      "title": "GitHub",
      "username": "meuusuario",
      "password": "criptografado...",
      "url": "https://github.com",
      "notes": "",
      "tags": ["dev"],
      "created_at": "2024-01-15T10:30:00Z",
      "modified_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

---

#### `GetEntry`

Recupera uma única entrada por ID ou correspondência de título.

**Requisição:**
```json
{
  "id": "req-5",
  "action": "GetEntry",
  "params": {
    "query": "GitHub"
  }
}
```

**Resposta:**
```json
{
  "id": "req-5",
  "ok": true,
  "data": {
    "id": 1,
    "title": "GitHub",
    "username": "meuusuario",
    "password": "senha-descriptografada",
    "url": "https://github.com",
    "notes": "",
    "tags": ["dev"],
    "created_at": "2024-01-15T10:30:00Z",
    "modified_at": "2024-01-15T10:30:00Z"
  }
}
```

---

#### `AddEntry`

Cria uma nova entrada no cofre.

**Requisição:**
```json
{
  "id": "req-6",
  "action": "AddEntry",
  "params": {
    "title": "Twitter",
    "username": "@meuhandle",
    "password": "minha-senha-segura",
    "url": "https://twitter.com",
    "notes": "Conta pessoal",
    "tags": ["social"]
  }
}
```

**Resposta:**
```json
{
  "id": "req-6",
  "ok": true,
  "data": {
    "id": 2
  }
}
```

---

#### `UpdateEntry`

Atualiza uma entrada existente.

**Requisição:**
```json
{
  "id": "req-7",
  "action": "UpdateEntry",
  "params": {
    "id": 2,
    "title": "X (Twitter)",
    "password": "nova-senha"
  }
}
```

**Resposta:**
```json
{
  "id": "req-7",
  "ok": true,
  "data": {
    "updated": true
  }
}
```

---

#### `DeleteEntry`

Exclui uma entrada por ID.

**Requisição:**
```json
{
  "id": "req-8",
  "action": "DeleteEntry",
  "params": {
    "id": 2
  }
}
```

**Resposta:**
```json
{
  "id": "req-8",
  "ok": true,
  "data": {
    "deleted": true
  }
}
```

---

### Ações do Gerador

Estas ações não exigem que o cofre esteja desbloqueado.

#### `GeneratePassword`

Gera uma senha aleatória ou passphrase.

**Requisição (senha):**
```json
{
  "id": "req-9",
  "action": "GeneratePassword",
  "params": {
    "length": 20,
    "symbols": true
  }
}
```

**Requisição (passphrase):**
```json
{
  "id": "req-10",
  "action": "GeneratePassword",
  "params": {
    "words": 6
  }
}
```

**Resposta:**
```json
{
  "id": "req-9",
  "ok": true,
  "data": {
    "password": "aB3$k9!mP2@qR7&xL4",
    "entropy_bits": 128,
    "score": 4,
    "label": "forte"
  }
}
```

---

#### `CheckPassword`

Analisa a força da senha sem armazená-la.

**Requisição:**
```json
{
  "id": "req-11",
  "action": "CheckPassword",
  "params": {
    "password": "senha-para-verificar"
  }
}
```

**Resposta:**
```json
{
  "id": "req-11",
  "ok": true,
  "data": {
    "entropy_bits": 52,
    "score": 2,
    "label": "média",
    "warning": "Esta é uma senha comum entre as 100 mais usadas"
  }
}
```

---

## Executando o Daemon

### Desenvolvimento

```bash
# Compila o daemon
cargo build -p vault-daemon

# Executa com configurações padrão
cargo run -p vault-daemon

# O daemon irá:
# 1. Carregar/criar configuração em ~/.config/vault/config.toml
# 2. Criar o socket em ~/.local/share/keyhaven/daemon.sock
# 3. Começar a escutar por conexões
```

### Produção

```bash
# Compila binário de release
cargo build --release -p vault-daemon

# Executa como serviço systemd (exemplo de arquivo de serviço)
systemctl --user enable vault-daemon
systemctl --user start vault-daemon
```

### Localização do Socket

Por padrão, o daemon cria seu socket em:
- `$XDG_RUNTIME_DIR/vault.sock` (se disponível)
- `/tmp/vault.sock` (fallback)

Os clientes devem verificar estes locais ou ler do arquivo de configuração.

---

## Tratamento de Erros

Respostas de erro comuns:

| Erro | Causa | Resolução |
|------|-------|-----------|
| `Cofre bloqueado. Execute: vault unlock` | Sessão expirou ou nunca foi desbloqueada | Chame `Unlock` com a senha mestre |
| `Senha mestre incorreta` | Senha errada durante o desbloqueio | Tente novamente com a senha correta |
| `Entrada não encontrada` | A consulta não correspondeu a nenhuma entrada | Verifique a consulta/id |
| `Campo 'X' é obrigatório` | Parâmetro ausente na requisição | Adicione o campo obrigatório |
| `Parâmetros inválidos: ...` | Erro de parsing JSON | Verifique o formato da requisição |

---

## Detalhes de Implementação

### Segurança de Threads

O daemon usa `tokio::sync::Mutex` para o estado da sessão:

- Múltiplas conexões são tratadas simultaneamente
- O estado da sessão é compartilhado entre todas as conexões
- A contenção de locks é minimizada (tarefa em segundo plano verifica a cada 30s)

### Limites de Tamanho de Mensagem

O daemon rejeita mensagens maiores que **1 MB** para prevenir ataques DoS.

### Encerramento Gracioso

O daemon remove seu arquivo de socket na inicialização se ele existir (lida com travamentos anteriores). Atualmente não há tratamento explícito de sinal de encerramento.

---

## Veja Também

- [Visão Geral da Arquitetura](../architecture.md) - Design geral do sistema
- [Detalhes de Criptografia](../crypto.md) - Criptografia e derivação de chaves
- [Guia da CLI](../cli/cli-pt-br.md) - Uso do cliente de linha de comando
