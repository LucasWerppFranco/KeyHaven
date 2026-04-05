-- migrations/001_initial.sql
-- SQLCipher: banco inteiro criptografado com AES-256-CBC
-- A chave é derivada via Argon2id a partir da senha mestre
-- NUNCA armazenamos a senha mestre ou a chave derivada em disco

-- ── Metadados do vault ──────────────────────────────────────────────────────
-- Uma única linha: guarda o salt do Argon2 e o HMAC de verificação.
-- O HMAC permite confirmar se a senha mestre está correta antes de
-- descriptografar qualquer entrada (evita descriptografar lixo silenciosamente).
CREATE TABLE IF NOT EXISTS vault_meta (
    id            INTEGER PRIMARY KEY CHECK (id = 1), -- garante exatamente 1 linha
    version       INTEGER NOT NULL DEFAULT 1,

    -- Salt aleatório (32 bytes) gerado na criação do vault, nunca muda
    -- Armazenado como BLOB binário, não como hex
    argon2_salt   BLOB    NOT NULL,

    -- Parâmetros do Argon2id usados na derivação
    -- Guardados aqui para poder aumentar no futuro sem invalidar o vault
    argon2_m_cost INTEGER NOT NULL DEFAULT 65536,  -- 64 MB de memória
    argon2_t_cost INTEGER NOT NULL DEFAULT 3,       -- 3 iterações
    argon2_p_cost INTEGER NOT NULL DEFAULT 4,       -- 4 threads paralelas

    -- HMAC-SHA256 de uma string conhecida ("vault-ok") com a chave derivada.
    -- Na abertura: re-deriva a chave, recalcula o HMAC, compara.
    -- Se bater → senha correta. Se não bater → senha errada.
    verification_hmac BLOB NOT NULL,

    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Entradas do vault ───────────────────────────────────────────────────────
-- Cada campo sensível é criptografado individualmente com AES-256-GCM.
-- Isso permite busca por campos não-sensíveis (title, url_host) sem
-- descriptografar tudo, e minimiza o impacto de um vazamento parcial.
CREATE TABLE IF NOT EXISTS entries (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Campos em texto claro (usados para busca e listagem rápida)
    -- O host é extraído da URL para facilitar o matching da extensão
    url_host      TEXT    NOT NULL DEFAULT '',  -- ex: "github.com"

    -- Campos criptografados individualmente (AES-256-GCM, nonce único por campo)
    -- Formato: nonce(12 bytes) || ciphertext || tag(16 bytes), tudo como BLOB
    enc_title     BLOB    NOT NULL,
    enc_username  BLOB    NOT NULL,
    enc_password  BLOB    NOT NULL,
    enc_url       BLOB    NOT NULL,
    enc_notes     BLOB,  -- nullable: nem toda entrada tem notas

    -- Timestamps em ISO 8601 UTC (não criptografados, usados para ordenação)
    created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT    NOT NULL DEFAULT (datetime('now')),

    -- Soft delete: marcamos como deletado em vez de remover
    -- Permite histórico e recuperação eventual
    deleted_at    TEXT    -- NULL = ativa, preenchido = deletada
);

-- ── Tags ────────────────────────────────────────────────────────────────────
-- Sistema simples de tags para organização (ex: "trabalho", "pessoal", "banco")
-- Tags em texto claro: são metadados de organização, não dados sensíveis
CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT    NOT NULL UNIQUE COLLATE NOCASE
);

CREATE TABLE IF NOT EXISTS entry_tags (
    entry_id  INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    tag_id    INTEGER NOT NULL REFERENCES tags(id)    ON DELETE CASCADE,
    PRIMARY KEY (entry_id, tag_id)
);

-- ── Histórico de senhas ─────────────────────────────────────────────────────
-- Toda vez que a senha de uma entrada muda, a anterior vai para cá.
-- Permite "recuperar senha antiga" sem expor o histórico na listagem normal.
CREATE TABLE IF NOT EXISTS password_history (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id      INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    enc_password  BLOB    NOT NULL,   -- mesmo formato: nonce || ciphertext || tag
    replaced_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ── Índices ─────────────────────────────────────────────────────────────────
-- Busca por host (usado pelo matching da extensão de browser)
CREATE INDEX IF NOT EXISTS idx_entries_url_host
    ON entries(url_host)
    WHERE deleted_at IS NULL;

-- Listagem por data de atualização (ordem padrão da UI)
CREATE INDEX IF NOT EXISTS idx_entries_updated_at
    ON entries(updated_at DESC)
    WHERE deleted_at IS NULL;

-- Histórico por entrada
CREATE INDEX IF NOT EXISTS idx_password_history_entry
    ON password_history(entry_id, replaced_at DESC);

-- ── Trigger: atualiza updated_at automaticamente ─────────────────────────────
CREATE TRIGGER IF NOT EXISTS entries_updated_at
    AFTER UPDATE ON entries
    FOR EACH ROW
BEGIN
    UPDATE entries SET updated_at = datetime('now') WHERE id = NEW.id;
END;
