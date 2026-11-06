//! Conexión SQLite, migraciones y repositorios.
//!
//! El pool vive en el estado de Tauri. Toda persistencia pasa por aquí:
//! el frontend nunca habla SQL.

use std::fs;
use std::path::PathBuf;

use chrono::Local;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const MIGRATIONS: &[(&str, &str)] = &[("0001_init", include_str!("../../migrations/0001_init.sql"))];

pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type DbConn = PooledConnection<SqliteConnectionManager>;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn conn(&self) -> Result<DbConn, String> {
        self.pool
            .get()
            .map_err(|error| format!("no hay conexión a la base de datos: {error}"))
    }
}

pub fn init(app: &AppHandle) -> Result<Database, String> {
    let path = database_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let manager = SqliteConnectionManager::file(&path).with_init(|connection| {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;",
        )?;
        Ok(())
    });

    let pool = Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(|error| error.to_string())?;

    {
        let connection = pool.get().map_err(|error| error.to_string())?;
        migrate(&connection)?;
    }

    Ok(Database { pool })
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../database/assistant.db");
        return Ok(path);
    }

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir.join("assistant.db"))
}

fn migrate(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .map_err(|error| error.to_string())?;

    for (version, sql) in MIGRATIONS {
        let already: Option<String> = connection
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                params![version],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;

        if already.is_some() {
            continue;
        }

        connection
            .execute_batch(sql)
            .map_err(|error| format!("migración {version}: {error}"))?;
        connection
            .execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![version, now()],
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn now() -> String {
    Local::now().to_rfc3339()
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ---------------------------------------------------------------------------
// Conversaciones y mensajes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub tokens_in: Option<i64>,
    pub tokens_out: Option<i64>,
    pub cost_usd: Option<f64>,
    pub status: String,
    pub created_at: String,
}

pub fn list_conversations(db: &Database) -> Result<Vec<Conversation>, String> {
    let conn = db.conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, archived, created_at, updated_at
             FROM conversations
             WHERE archived = 0
             ORDER BY updated_at DESC",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                archived: row.get::<_, i64>(2)? != 0,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_conversation(db: &Database, title: Option<String>) -> Result<Conversation, String> {
    let conn = db.conn()?;
    let id = new_id();
    let stamp = now();
    let title = title.unwrap_or_else(|| "Nueva conversación".to_string());

    conn.execute(
        "INSERT INTO conversations (id, title, archived, created_at, updated_at)
         VALUES (?1, ?2, 0, ?3, ?3)",
        params![id, title, stamp],
    )
    .map_err(|error| error.to_string())?;

    Ok(Conversation {
        id,
        title,
        archived: false,
        created_at: stamp.clone(),
        updated_at: stamp,
    })
}

pub fn get_conversation(db: &Database, id: &str) -> Result<Option<Conversation>, String> {
    let conn = db.conn()?;
    conn.query_row(
        "SELECT id, title, archived, created_at, updated_at FROM conversations WHERE id = ?1",
        params![id],
        |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                archived: row.get::<_, i64>(2)? != 0,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())
}

pub fn rename_conversation(db: &Database, id: &str, title: &str) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now(), id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn touch_conversation(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![now(), id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn archive_conversation(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE conversations SET archived = 1, updated_at = ?1 WHERE id = ?2",
        params![now(), id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_messages(db: &Database, conversation_id: &str) -> Result<Vec<ChatMessage>, String> {
    let conn = db.conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, model, tokens_in, tokens_out,
                    cost_usd, status, created_at
             FROM messages
             WHERE conversation_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![conversation_id], row_to_message)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn row_to_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatMessage> {
    Ok(ChatMessage {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        model: row.get(4)?,
        tokens_in: row.get(5)?,
        tokens_out: row.get(6)?,
        cost_usd: row.get(7)?,
        status: row.get(8)?,
        created_at: row.get(9)?,
    })
}

pub fn insert_message(
    db: &Database,
    conversation_id: &str,
    role: &str,
    content: &str,
    model: Option<&str>,
    status: &str,
) -> Result<ChatMessage, String> {
    let conn = db.conn()?;
    let message = ChatMessage {
        id: new_id(),
        conversation_id: conversation_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        model: model.map(str::to_string),
        tokens_in: None,
        tokens_out: None,
        cost_usd: None,
        status: status.to_string(),
        created_at: now(),
    };

    conn.execute(
        "INSERT INTO messages (
            id, conversation_id, role, content, model, tokens_in, tokens_out,
            cost_usd, status, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL, NULL, ?6, ?7)",
        params![
            message.id,
            message.conversation_id,
            message.role,
            message.content,
            message.model,
            message.status,
            message.created_at
        ],
    )
    .map_err(|error| error.to_string())?;

    touch_conversation(db, conversation_id)?;
    Ok(message)
}

pub fn finalize_message(
    db: &Database,
    id: &str,
    content: &str,
    tokens_in: i64,
    tokens_out: i64,
    cost_usd: f64,
    status: &str,
) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE messages
         SET content = ?1, tokens_in = ?2, tokens_out = ?3, cost_usd = ?4, status = ?5
         WHERE id = ?6",
        params![content, tokens_in, tokens_out, cost_usd, status, id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn delete_message(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute("DELETE FROM messages WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Ajustes
// ---------------------------------------------------------------------------

pub fn get_setting(db: &Database, key: &str) -> Result<Option<String>, String> {
    let conn = db.conn()?;
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

pub fn set_setting(db: &Database, key: &str, value: &str) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Uso / coste
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsage {
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_usd: f64,
    pub date: String,
}

pub fn log_usage(
    db: &Database,
    conversation_id: &str,
    message_id: &str,
    model: &str,
    tokens_in: i64,
    tokens_out: i64,
    cost_usd: f64,
) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO usage_log (
            id, conversation_id, message_id, model, tokens_in, tokens_out, cost_usd, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            new_id(),
            conversation_id,
            message_id,
            model,
            tokens_in,
            tokens_out,
            cost_usd,
            now()
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn daily_usage(db: &Database) -> Result<DailyUsage, String> {
    let conn = db.conn()?;
    let date = Local::now().format("%Y-%m-%d").to_string();

    let (tokens_in, tokens_out, cost_usd): (i64, i64, f64) = conn
        .query_row(
            "SELECT COALESCE(SUM(tokens_in), 0),
                    COALESCE(SUM(tokens_out), 0),
                    COALESCE(SUM(cost_usd), 0)
             FROM usage_log
             WHERE substr(created_at, 1, 10) = ?1",
            params![date],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|error| error.to_string())?;

    Ok(DailyUsage {
        tokens_in,
        tokens_out,
        cost_usd,
        date,
    })
}
