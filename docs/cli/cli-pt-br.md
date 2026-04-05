# CLI do KeyHaven

A **vault-cli** (nome do binário: `keyhaven`) é a interface de linha de comando do gerenciador de senhas KeyHaven. Ela serve como prova de conceito para a biblioteca `vault-core` antes de construir interfaces visuais.

## Índice

- [Visão Geral](#visão-geral)
- [Instalação](#instalação)
- [Comandos](#comandos)
- [Saída para Pipes](#saída-para-pipes)
- [Integração com Rofi/Wofi](#integração-com-rofiwofi)
- [Recursos de Segurança](#recursos-de-segurança)

---

## Visão Geral

A CLI fornece gerenciamento completo do cofre através de uma estrutura de comandos intuitiva. Ela utiliza:

- **clap** com macros derive para estrutura de comandos e geração automática de `--help`
- **rpassword** para entrada segura de senhas (sem eco no terminal)
- **comfy-table** para tabelas formatadas
- **indicatif** para spinners de progresso
- **colored** para saída colorida no terminal

---

## Instalação

A CLI é construída como parte do workspace:

```bash
cargo build --package vault-cli
```

O binário está disponível em `./target/debug/keyhaven`.

---

## Comandos

### `init`

Inicializa um novo cofre com uma senha mestre.

```bash
keyhaven init
```

- Cria o banco de dados em `~/.config/keyhaven/vault.db` (ou `--db-path` especificado)
- Solicita a senha mestre (mínimo 12 caracteres)
- Confirma a senha para evitar erros de digitação

---

### `unlock`

Desbloqueia o cofre e armazena a chave derivada em memória.

```bash
keyhaven unlock [--timeout 15m]
```

**Opções:**
- `--timeout`: Tempo limite da sessão (formato: `30s`, `15m`, `1h`). Padrão: `15m`

**Nota:** Nesta prova de conceito, a chave é armazenada em uma variável de ambiente (`KEYHAVEN_SESSION_KEY`). Em produção, o daemon manteria a chave.

---

### `lock`

Bloqueia imediatamente o cofre, limpando a chave da sessão da memória.

```bash
keyhaven lock
```

---

### `list`

Lista todas as entradas de senha.

```bash
keyhaven list [--search <consulta>] [--json]
```

**Opções:**
- `-s, --search`: Filtra entradas por título/usuário
- `--json`: Saída em JSON para scripts

**Saída interativa:**
```
╭────┬─────────────┬────────────┬─────────────────┬────────────────╮
│ ID │ Titulo      │ Usuario    │ URL             │ Modificado     │
├────┼─────────────┼────────────┼─────────────────┼────────────────┤
│ 1  │ GitHub      │ usuario... │ github.com      │ 04/04/2025...  │
╰────┴─────────────┴────────────┴─────────────────┴────────────────╯
```

**Saída para pipes:**
```
GitHub	usuario@exemplo.com	github.com
GitLab	usuario@exemplo.com	gitlab.com
```

---

### `get`

Recupera e exibe uma entrada de senha.

```bash
keyhaven get <consulta> [--copy] [--show] [--field <campo>]
```

**Opções:**
- `--copy`: Copia a senha para o clipboard (limpa após 30s)
- `--show`: Exibe a senha em texto limpo
- `--field`: Exibe apenas o campo especificado (para pipes)

**Exemplos:**
```bash
# Mostra detalhes da entrada
keyhaven get github

# Copia a senha para o clipboard
keyhaven get github --copy

# Exibe apenas a senha (para scripts)
keyhaven get github --field senha | wl-copy

# Exibe apenas o usuário
keyhaven get github --field usuario
```

**Campos disponíveis:** `senha`, `usuario`, `titulo`, `url`, `notas` (ou `password`, `username`, `title`, `notes`)

---

### `add`

Adiciona interativamente uma nova entrada de senha.

```bash
keyhaven add [--url <url>] [--gen]
```

**Opções:**
- `--url`: Pré-preenche o campo URL
- `--gen`: Gera senha automaticamente

**Prompts interativos:**
- Título (obrigatório)
- Usuário (opcional)
- Senha (ou auto-gerada com `--gen`)
- URL (opcional)
- Notas (opcional)

---

### `gen`

Gera uma senha segura ou passphrase.

```bash
keyhaven gen [--length 20] [--symbols] [--copy]
keyhaven gen [--words 4] [--copy]
```

**Opções:**
- `-l, --length`: Comprimento da senha (padrão: 20)
- `--symbols`: Inclui caracteres especiais
- `--words`: Gera uma passphrase estilo Diceware (N palavras)
- `--copy`: Copia o resultado para o clipboard

**Exemplos:**
```bash
# Gera senha de 20 caracteres
keyhaven gen

# Gera passphrase com 6 palavras
keyhaven gen --words 6

# Gera e copia
keyhaven gen --length 32 --symbols --copy
```

---

### `check`

Verifica a força da senha e status de vazamentos via Have I Been Pwned.

```bash
keyhaven check <senha>
```

**Recursos:**
- Análise local de força (cálculo de entropia)
- Verificação de vazamentos HIBP usando k-anonymity

**Exemplo de saída:**
```
Verificando senha...

✓ Forca da senha: Forte

🔍 Verificando HIBP...
⚠ Senha encontrada em 49141 vazamentos!
   Esta senha nao e segura. Mude imediatamente!
```

**Nota:** A API HIBP usa k-anonymity — apenas os primeiros 5 caracteres do hash SHA-1 são enviados ao servidor.

---

### `rofi`

Abre um seletor rofi/wofi para integração com Hyprland/ambiente desktop.

```bash
keyhaven rofi [--type]
```

**Opções:**
- `--type`: Digita a senha usando `ydotool` em vez do clipboard

**Configuração do Hyprland:**
```
bind = SUPER, P, exec, keyhaven rofi
bind = SUPER SHIFT, P, exec, keyhaven rofi --type
```

**Comportamento:**
- Lista todas as entradas no rofi/wofi
- Copia a senha selecionada para o clipboard
- O clipboard é limpo após 30 segundos
- Fallback para clipboard se `ydotool` não estiver disponível

---

## Saída para Pipes

Quando a flag `--field` é usada ou a saída é redirecionada (não é um TTY), a CLI produz saída limpa sem cores, ícones ou texto decorativo — apenas o valor bruto.

**Detecção:**
```rust
use std::io::IsTerminal;

if std::io::stdout().is_terminal() {
    // Interativo: usa cores e formatação
} else {
    // Pipe: saída limpa
}
```

**Exemplo:**
```bash
# Em script: copia apenas a senha
keyhaven get github --field senha | wl-copy

# Saída: apenas a senha, sem newline extra
minhasenha123
```

---

## Integração com Rofi/Wofi

O comando `rofi` fornece integração com o ambiente desktop Linux:

1. **Listagem:** Formata entradas como `titulo: usuario` para o launcher
2. **Seleção:** Captura o índice selecionado do rofi/wofi
3. **Ação:** Copia ou digita a senha
4. **Limpeza:** Agenda a limpeza do clipboard após 30 segundos

**Requisitos:**
- `rofi` ou `wofi` instalado
- `wl-copy` (Wayland) ou `xclip` (X11) para o clipboard
- `ydotool` (opcional) para modo de digitação

---

## Recursos de Segurança

| Recurso          | Implementação                                    |
|------------------|--------------------------------------------------|
| Entrada de senha | `rpassword` — sem eco no terminal                  |
| Chave de sessão  | Armazenada com wrapper `Zeroizing` (limpa no drop) |
| Clipboard        | Auto-limpo após 30 segundos                      |
| Verificação HIBP | k-anonymity (apenas 5 chars do SHA-1 enviados)   |
| Saída para pipes | Limpa, sem vazamento de metadados                |
| Saída colorida   | Desativada quando não é TTY                      |

---

## Opções Globais

Todos os comandos suportam:

- `-d, --db-path <CAMINHO>`: Caminho customizado do banco de dados
- `-s, --socket-path <CAMINHO>`: Caminho customizado do socket do daemon

**Exemplos:**
```bash
keyhaven -d /tmp/test.db init
keyhaven -d /tmp/test.db -s /tmp/test.sock unlock
```
