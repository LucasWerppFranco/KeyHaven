# Como o KeyHaven funciona (para usuários)

**O KeyHaven** é um gerenciador de senhas que funciona como um cofre digital com fechadura inteligente. Veja como usá-lo no dia a dia:

---

## 1. Configuração (Uma Única Vez)

Ao executar o KeyHaven pela primeira vez, você cria um **cofre** — um banco de dados criptografado onde todas as suas senhas são armazenadas:

```bash
# Inicialize seu cofre com uma senha mestra
keyhaven init
# Digite sua senha mestra: **********
```

Isso cria:
- Um arquivo de banco de dados criptografado (`~/.local/share/vault/vault.db`)
- Um arquivo de configuração (`~/.config/vault/config.toml`)
- O **daemon** começa a ser executado em segundo plano

---

## 2. Fluxo de Trabalho de Uso Diário

### Desbloqueando o Cofre

Antes de acessar qualquer senha, você deve **desbloquear** o cofre:

```bash
keyhaven unlock
# Digite a senha mestra: **********
# Cofre desbloqueado (bloqueia automaticamente após 15 minutos de uso) inatividade)
```

Ao desbloquear:
- Sua senha mestra nunca é armazenada
- Em vez disso, o daemon deriva uma chave de criptografia e a mantém na memória
- A chave é apagada da memória se o daemon travar ou for encerrado

### Usando suas senhas

Uma vez desbloqueado, você pode:

```bash
# Adicionar uma nova senha
keyhaven add github
# Nome de usuário: myuser
# Senha: ********
# URL: https://github.com

# Recuperar uma senha (copiada para a área de transferência)
keyhaven get github
# Senha copiada para a área de transferência (apagada após 30 segundos)

# Listar todas as entradas
keyhaven list
# Pesquisar: "git"
# 1. github - myuser

# Gerar uma senha forte
keyhaven generate --length 20
# Gerada: aB3$k9!mP2@qR7&xL4
```

### Bloqueando

O cofre **Bloqueia automaticamente** após 15 minutos de inatividade. Ou você pode bloqueá-lo manualmente:

```bash
keyhaven lock
```

Quando bloqueado, a chave de criptografia é apagada da memória. Suas senhas ficam inacessíveis até que você desbloqueie novamente.

---

## 3. O Modelo de Segurança

**Por que um daemon?** Em vez de cada comando carregar o cofre do disco (lento e arriscado), o daemon mantém o cofre pronto na memória, preservando a segurança:

```
┌─────────────┐    bloqueado    ┌─────────────┐
│ Você        │ ──────────────→ │Criptografado│
│             │                 │Banco de Dados
│             │ ←────────────── │(Cofre)      │
└─────────────┘   desbloquear   └─────────────┘
        ↓                            ↑
        │                            │
        └──────→ Daemon ←────────────┘
              (Segura a chave) 
        (somente quando desbloqueado)
```

**Principais recursos de segurança:**
- **Senha mestra** → Nunca armazenada em nenhum lugar. Usada apenas para derivar a chave de criptografia.

- **Argon2id** → Derivação de chave padrão da indústria que resiste a ataques de força bruta (leva cerca de 100 ms para verificar)
- **AES-256-GCM** → Criptografia de nível militar para cada entrada de senha
- **Bloqueio automático** → A chave é apagada da RAM após inatividade (evita despejos de memória)
- **Socket Unix** → Somente sua conta de usuário pode se comunicar com o daemon

---

## 4. Múltiplas formas de acesso

O daemon aceita conexões de:
- **CLI** (comando `keyhaven`)
- **Aplicativos GUI** (podem ser criados para se conectar via socket)
- **Extensões de navegador** (via mensagens nativas)

Todos se comunicam com o mesmo daemon, portanto, desbloquear em um desbloqueia em todos.

---

## 5. O que acontece quando...

| Cenário | O que acontece |

|----------|-------------|
| O computador entra em modo de suspensão | O cofre permanece destrancado (chave na memória) |

| O daemon trava | O cofre bloqueia imediatamente (chave perdida da RAM) |

| Senha incorreta | Leva cerca de 100 ms para falhar (impede tentativas de adivinhação) |

| Copiar senha | Limpa da área de transferência após 30 segundos |

| Bloqueio automático acionado | Chave zerada na memória, você precisa desbloquear novamente |

---

## Resumo

O KeyHaven funciona como um cofre físico com trava de tempo:

1. **Inicializar uma vez** → Cria um cofre criptografado
2. **Desbloquear** → O cofre abre, a chave permanece apenas na memória
3. **Usar** → Adicionar/recuperar senhas instantaneamente
4. **Bloqueio automático** → O cofre fecha após inatividade, a chave é destruída

Suas senhas são criptografadas em repouso (no arquivo) e descriptografadas apenas temporariamente enquanto o cofre estiver destrancado. A senha mestra nunca sai da sua cabeça.
