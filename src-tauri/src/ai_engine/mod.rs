//! Cliente de IA compatible con OpenAI y Ollama, streaming y control de coste.

mod client;
mod pricing;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::{AppHandle, State};

use crate::db::{self, ChatMessage, Conversation, DailyUsage, Database};
use crate::secrets;

const SETTINGS_KEY: &str = "ai.settings";
const KEY_REF_PREFIX: &str = "ai.api_key.";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub title_model: String,
    pub temperature: f32,
    pub daily_spend_limit_usd: f64,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            title_model: "gpt-4o-mini".to_string(),
            temperature: 0.7,
            daily_spend_limit_usd: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsView {
    #[serde(flatten)]
    pub settings: AiSettings,
    pub has_api_key: bool,
    pub key_required: bool,
}

pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

struct ActiveGeneration {
    id: String,
    cancelled: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct AiRuntime {
    current: Mutex<Option<ActiveGeneration>>,
    http: reqwest::Client,
}

impl AiRuntime {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent("Asistente-DazNova/0.1")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("cliente HTTP");
        Self {
            current: Mutex::new(None),
            http,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StreamEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        generation_id: String,
        conversation_id: String,
        user_message: ChatMessage,
        assistant_message_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Token { text: String },
    #[serde(rename_all = "camelCase")]
    Done { message: ChatMessage, usage: DailyUsage },
    #[serde(rename_all = "camelCase")]
    Cancelled { message: ChatMessage },
    #[serde(rename_all = "camelCase")]
    Error { message: String },
}

fn system_prompt() -> String {
    let now = Local::now().format("%Y-%m-%d %H:%M");
    format!(
        "Eres DazNova, un asistente personal de escritorio. Eres claro, útil y conciso.\n\
         Ayudas con la jornada diaria, recuerdas el contexto y no inventas datos que no tienes.\n\
         Fecha y hora actuales: {now} (hora local del usuario).\n\
         Responde en el idioma en el que te hablen."
    )
}

fn key_ref(provider: &str) -> String {
    format!("{KEY_REF_PREFIX}{provider}")
}

fn load_settings(db: &Database) -> Result<AiSettings, String> {
    match db::get_setting(db, SETTINGS_KEY)? {
        Some(raw) => serde_json::from_str(&raw).map_err(|error| error.to_string()),
        None => Ok(AiSettings::default()),
    }
}

fn apply_provider_defaults(settings: &mut AiSettings) {
    match settings.provider.as_str() {
        "openai" => {
            if settings.base_url.is_empty()
                || settings.base_url.contains("11434")
                || settings.base_url.contains("openrouter")
            {
                settings.base_url = "https://api.openai.com/v1".to_string();
            }
        }
        "ollama" => {
            if settings.base_url.is_empty() || settings.base_url.contains("openai.com") {
                settings.base_url = "http://127.0.0.1:11434/v1".to_string();
            }
            if settings.model.contains("gpt-") {
                settings.model = "llama3.2".to_string();
            }
            if settings.title_model.contains("gpt-") {
                settings.title_model = settings.model.clone();
            }
        }
        _ => {}
    }
    if !(0.0..=2.0).contains(&settings.temperature) {
        settings.temperature = 0.7;
    }
    if settings.daily_spend_limit_usd < 0.0 {
        settings.daily_spend_limit_usd = 0.0;
    }
}

fn key_required(provider: &str) -> bool {
    provider != "ollama"
}

fn api_key_for(settings: &AiSettings) -> Result<Option<String>, String> {
    secrets::get_secret(&key_ref(&settings.provider))
}

// ---------------------------------------------------------------------------
// Comandos
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_ai_settings(db: State<'_, Database>) -> Result<AiSettingsView, String> {
    let mut settings = load_settings(&db)?;
    apply_provider_defaults(&mut settings);
    Ok(AiSettingsView {
        has_api_key: secrets::has_secret(&key_ref(&settings.provider))?,
        key_required: key_required(&settings.provider),
        settings,
    })
}

#[tauri::command]
pub fn save_ai_settings(db: State<'_, Database>, mut settings: AiSettings) -> Result<AiSettingsView, String> {
    settings.provider = settings.provider.trim().to_ascii_lowercase();
    settings.base_url = settings.base_url.trim().to_string();
    settings.model = settings.model.trim().to_string();
    settings.title_model = settings.title_model.trim().to_string();
    apply_provider_defaults(&mut settings);

    let raw = serde_json::to_string(&settings).map_err(|error| error.to_string())?;
    db::set_setting(&db, SETTINGS_KEY, &raw)?;
    get_ai_settings(db)
}

#[tauri::command]
pub fn save_api_key(db: State<'_, Database>, api_key: String) -> Result<AiSettingsView, String> {
    let settings = load_settings(&db)?;
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        secrets::delete_secret(&key_ref(&settings.provider))?;
    } else {
        secrets::set_secret(&key_ref(&settings.provider), trimmed)?;
    }
    get_ai_settings(db)
}

#[tauri::command]
pub async fn validate_api_key(
    db: State<'_, Database>,
    runtime: State<'_, AiRuntime>,
    api_key: Option<String>,
) -> Result<(), String> {
    let mut settings = load_settings(&db)?;
    apply_provider_defaults(&mut settings);
    let stored = api_key_for(&settings)?;
    let key = api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or(stored);

    if key_required(&settings.provider) && key.is_none() {
        return Err("Guarda una API key antes de comprobar la conexión.".to_string());
    }

    client::validate_endpoint(&runtime.http, &settings, key.as_deref()).await
}

#[tauri::command]
pub fn get_daily_usage(db: State<'_, Database>) -> Result<DailyUsage, String> {
    db::daily_usage(&db)
}

#[tauri::command]
pub fn list_conversations(db: State<'_, Database>) -> Result<Vec<Conversation>, String> {
    db::list_conversations(&db)
}

#[tauri::command]
pub fn create_conversation(db: State<'_, Database>) -> Result<Conversation, String> {
    db::create_conversation(&db, None)
}

#[tauri::command]
pub fn archive_conversation(db: State<'_, Database>, conversation_id: String) -> Result<(), String> {
    db::archive_conversation(&db, &conversation_id)
}

#[tauri::command]
pub fn list_messages(
    db: State<'_, Database>,
    conversation_id: String,
) -> Result<Vec<ChatMessage>, String> {
    db::list_messages(&db, &conversation_id)
}

#[tauri::command]
pub fn cancel_generation(runtime: State<'_, AiRuntime>, generation_id: String) -> Result<(), String> {
    let current = runtime.current.lock().map_err(|error| error.to_string())?;
    if let Some(active) = current.as_ref() {
        if active.id == generation_id {
            active.cancelled.store(true, Ordering::Relaxed);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    db: State<'_, Database>,
    runtime: State<'_, AiRuntime>,
    conversation_id: Option<String>,
    content: String,
    on_event: Channel<StreamEvent>,
) -> Result<(), String> {
    let content = content.trim().to_string();
    if content.is_empty() {
        return Err("El mensaje está vacío.".to_string());
    }

    let mut settings = load_settings(&db)?;
    apply_provider_defaults(&mut settings);

    if key_required(&settings.provider) && !secrets::has_secret(&key_ref(&settings.provider))? {
        return Err("Falta la API key. Ábrela en Ajustes antes de conversar.".to_string());
    }

    let usage = db::daily_usage(&db)?;
    if usage.cost_usd >= settings.daily_spend_limit_usd && settings.daily_spend_limit_usd > 0.0 {
        return Err(format!(
            "Has alcanzado el límite diario de {limit:.2} USD.",
            limit = settings.daily_spend_limit_usd
        ));
    }

    let conversation = match conversation_id {
        Some(id) => db::get_conversation(&db, &id)?
            .ok_or_else(|| "la conversación no existe".to_string())?,
        None => db::create_conversation(&db, None)?,
    };

    let user_message = db::insert_message(&db, &conversation.id, "user", &content, None, "complete")?;
    let assistant_placeholder = db::insert_message(
        &db,
        &conversation.id,
        "assistant",
        "",
        Some(&settings.model),
        "streaming",
    )?;

    let mut history = vec![ChatTurn {
        role: "system".to_string(),
        content: system_prompt(),
    }];
    for message in db::list_messages(&db, &conversation.id)? {
        if message.id == assistant_placeholder.id {
            continue;
        }
        if message.role == "assistant" && message.status == "error" {
            continue;
        }
        if message.content.is_empty() {
            continue;
        }
        history.push(ChatTurn {
            role: message.role,
            content: message.content,
        });
    }

    let cancelled = Arc::new(AtomicBool::new(false));
    let generation_id = uuid::Uuid::new_v4().to_string();
    {
        let mut current = runtime.current.lock().map_err(|error| error.to_string())?;
        if let Some(previous) = current.as_ref() {
            previous.cancelled.store(true, Ordering::Relaxed);
        }
        *current = Some(ActiveGeneration {
            id: generation_id.clone(),
            cancelled: cancelled.clone(),
        });
    }

    let started = StreamEvent::Started {
        generation_id: generation_id.clone(),
        conversation_id: conversation.id.clone(),
        user_message: user_message.clone(),
        assistant_message_id: assistant_placeholder.id.clone(),
    };
    on_event.send(started).map_err(|error| error.to_string())?;

    let api_key = api_key_for(&settings)?;
    let http = runtime.http.clone();
    let assistant_id = assistant_placeholder.id.clone();
    let conversation_id = conversation.id.clone();
    let model = settings.model.clone();
    let was_untitled = conversation.title == "Nueva conversación";

    let channel = on_event.clone();
    let result = client::stream_chat(
        &http,
        &settings,
        api_key.as_deref(),
        &history,
        cancelled.clone(),
        |token| {
            let _ = channel.send(StreamEvent::Token {
                text: token.to_string(),
            });
        },
    )
    .await;

    {
        let mut current = runtime.current.lock().map_err(|error| error.to_string())?;
        if current.as_ref().is_some_and(|active| active.id == generation_id) {
            *current = None;
        }
    }

    match result {
        Ok(outcome) => {
            if outcome.text.is_empty() && outcome.cancelled {
                db::delete_message(&db, &assistant_id)?;
                on_event
                    .send(StreamEvent::Cancelled {
                        message: ChatMessage {
                            content: String::new(),
                            status: "cancelled".to_string(),
                            ..assistant_placeholder
                        },
                    })
                    .map_err(|error| error.to_string())?;
                return Ok(());
            }

            let tokens_in = outcome
                .usage
                .as_ref()
                .map(|usage| usage.prompt_tokens)
                .unwrap_or_else(|| pricing::estimate_tokens(&history.iter().map(|turn| turn.content.as_str()).collect::<Vec<_>>().join("\n")));
            let tokens_out = outcome
                .usage
                .as_ref()
                .map(|usage| usage.completion_tokens)
                .unwrap_or_else(|| pricing::estimate_tokens(&outcome.text));
            let cost = if settings.provider == "ollama" {
                0.0
            } else {
                pricing::estimate(&model, tokens_in, tokens_out)
            };
            let status = if outcome.cancelled { "cancelled" } else { "complete" };

            db::finalize_message(
                &db,
                &assistant_id,
                &outcome.text,
                tokens_in,
                tokens_out,
                cost,
                status,
            )?;
            db::log_usage(
                &db,
                &conversation_id,
                &assistant_id,
                &model,
                tokens_in,
                tokens_out,
                cost,
            )?;
            db::touch_conversation(&db, &conversation_id)?;

            let mut message = assistant_placeholder;
            message.content = outcome.text;
            message.tokens_in = Some(tokens_in);
            message.tokens_out = Some(tokens_out);
            message.cost_usd = Some(cost);
            message.status = status.to_string();
            message.model = Some(model.clone());

            if outcome.cancelled {
                on_event
                    .send(StreamEvent::Cancelled { message })
                    .map_err(|error| error.to_string())?;
            } else {
                let usage = db::daily_usage(&db)?;
                on_event
                    .send(StreamEvent::Done { message, usage })
                    .map_err(|error| error.to_string())?;

                if was_untitled {
                    let handle = app.clone();
                    let title_settings = settings.clone();
                    let title_key = api_key.clone();
                    let user_text = content.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = auto_title(handle, title_settings, title_key, conversation_id, user_text)
                            .await;
                    });
                }
            }
            Ok(())
        }
        Err(error) => {
            db::finalize_message(&db, &assistant_id, "", 0, 0, 0.0, "error")?;
            on_event
                .send(StreamEvent::Error {
                    message: error.clone(),
                })
                .map_err(|send_error| send_error.to_string())?;
            Err(error)
        }
    }
}

async fn auto_title(
    app: AppHandle,
    settings: AiSettings,
    api_key: Option<String>,
    conversation_id: String,
    user_text: String,
) -> Result<(), String> {
    let db = app.state::<Database>();
    let runtime = app.state::<AiRuntime>();
    let prompt = format!(
        "Resume esta conversación en un título de 4 a 6 palabras, sin comillas ni puntuación final:\n{user_text}"
    );
    let messages = vec![
        serde_json::json!({"role": "system", "content": "Devuelves solo un título corto."}),
        serde_json::json!({"role": "user", "content": prompt}),
    ];

    let title = client::complete_once(
        &runtime.http,
        &settings,
        api_key.as_deref(),
        &settings.title_model,
        messages,
    )
    .await?;

    let title: String = title.chars().take(80).collect();
    if title.chars().count() >= 3 {
        db::rename_conversation(&db, &conversation_id, &title)?;
    }
    Ok(())
}
