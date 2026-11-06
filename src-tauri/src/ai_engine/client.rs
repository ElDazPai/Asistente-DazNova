//! Cliente HTTP compatible con `/v1/chat/completions` (OpenAI, Ollama, Groq…).

use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::{AiSettings, ChatTurn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
}

pub struct StreamOutcome {
    pub text: String,
    pub usage: Option<CompletionUsage>,
    pub cancelled: bool,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Value],
    temperature: f32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
}

pub async fn stream_chat(
    http: &reqwest::Client,
    settings: &AiSettings,
    api_key: Option<&str>,
    history: &[ChatTurn],
    cancelled: Arc<AtomicBool>,
    mut on_token: impl FnMut(&str),
) -> Result<StreamOutcome, String> {
    let url = format!("{}/chat/completions", settings.base_url.trim_end_matches('/'));
    let messages = build_messages(history);
    let include_usage = settings.provider != "ollama";

    let body = ChatRequest {
        model: &settings.model,
        messages: &messages,
        temperature: settings.temperature,
        stream: true,
        stream_options: include_usage.then_some(StreamOptions {
            include_usage: true,
        }),
    };

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Asistente-DazNova/0.1"),
    );
    if let Some(key) = api_key.filter(|value| !value.is_empty()) {
        let value = format!("Bearer {key}");
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&value).map_err(|error| error.to_string())?,
        );
    }

    let response = http
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(map_transport)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(map_http_error(status.as_u16(), &body));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut text = String::new();
    let mut usage = None;

    while let Some(chunk) = stream.next().await {
        if cancelled.load(Ordering::Relaxed) {
            return Ok(StreamOutcome {
                text,
                usage,
                cancelled: true,
            });
        }

        let chunk = chunk.map_err(|error| format!("se cortó el flujo de respuesta: {error}"))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(index) = buffer.find('\n') {
            let mut line: String = buffer.drain(..=index).collect();
            if line.ends_with('\n') {
                line.pop();
            }
            if line.ends_with('\r') {
                line.pop();
            }
            if line.is_empty() {
                continue;
            }
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data.is_empty() {
                continue;
            }
            if data == "[DONE]" {
                return Ok(StreamOutcome {
                    text,
                    usage,
                    cancelled: false,
                });
            }

            let parsed: Value = match serde_json::from_str(data) {
                Ok(value) => value,
                Err(_) => continue,
            };

            if let Some(message) = parsed
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
            {
                return Err(map_api_message(message));
            }

            if let Some(delta) = parsed
                .pointer("/choices/0/delta/content")
                .and_then(Value::as_str)
            {
                if !delta.is_empty() {
                    text.push_str(delta);
                    on_token(delta);
                }
            }

            if let Some(full) = parsed
                .pointer("/choices/0/message/content")
                .and_then(Value::as_str)
            {
                if text.is_empty() && !full.is_empty() {
                    text.push_str(full);
                    on_token(full);
                }
            }

            if let Some(prompt) = parsed.pointer("/usage/prompt_tokens").and_then(Value::as_i64)
            {
                let completion = parsed
                    .pointer("/usage/completion_tokens")
                    .and_then(Value::as_i64)
                    .unwrap_or(0);
                usage = Some(CompletionUsage {
                    prompt_tokens: prompt,
                    completion_tokens: completion,
                });
            }
        }
    }

    Ok(StreamOutcome {
        text,
        usage,
        cancelled: cancelled.load(Ordering::Relaxed),
    })
}

pub async fn complete_once(
    http: &reqwest::Client,
    settings: &AiSettings,
    api_key: Option<&str>,
    model: &str,
    messages: Vec<Value>,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", settings.base_url.trim_end_matches('/'));

    let mut request = http.post(url).json(&serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": 0.2,
        "stream": false,
        "max_tokens": 48,
    }));

    if let Some(key) = api_key.filter(|value| !value.is_empty()) {
        request = request.bearer_auth(key);
    }

    let response = request.send().await.map_err(map_transport)?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(map_http_error(status.as_u16(), &body));
    }

    let parsed: Value = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    parsed
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(|text| text.trim().trim_matches('"').to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| "el modelo no devolvió un título".to_string())
}

pub async fn validate_endpoint(
    http: &reqwest::Client,
    settings: &AiSettings,
    api_key: Option<&str>,
) -> Result<(), String> {
    let url = format!("{}/models", settings.base_url.trim_end_matches('/'));
    let mut request = http.get(url);
    if let Some(key) = api_key.filter(|value| !value.is_empty()) {
        request = request.bearer_auth(key);
    }

    let response = request.send().await.map_err(map_transport)?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let body = response.text().await.unwrap_or_default();
    Err(map_http_error(status.as_u16(), &body))
}

fn build_messages(history: &[ChatTurn]) -> Vec<Value> {
    history
        .iter()
        .map(|turn| {
            serde_json::json!({
                "role": turn.role,
                "content": turn.content,
            })
        })
        .collect()
}

fn map_transport(error: reqwest::Error) -> String {
    if error.is_connect() || error.is_timeout() {
        if error.is_connect() {
            "Sin conexión. Comprueba la red o que Ollama esté en marcha.".to_string()
        } else {
            "La respuesta tardó demasiado. Inténtalo de nuevo.".to_string()
        }
    } else {
        format!("No se pudo contactar con el proveedor: {error}")
    }
}

fn map_http_error(status: u16, body: &str) -> String {
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    let message = parsed
        .as_ref()
        .and_then(|value| value.pointer("/error/message"))
        .and_then(Value::as_str)
        .unwrap_or(body);

    match status {
        401 | 403 => "API key inválida. Revísala en Ajustes.".to_string(),
        429 => "Límite de velocidad. Espera un momento y vuelve a intentar.".to_string(),
        _ if message.to_ascii_lowercase().contains("context")
            || message.to_ascii_lowercase().contains("maximum") =>
        {
            "El contexto es demasiado largo. Empieza una conversación nueva.".to_string()
        }
        _ => {
            let trimmed = message.trim();
            if trimmed.is_empty() {
                format!("El proveedor respondió {status}.")
            } else {
                map_api_message(trimmed)
            }
        }
    }
}

fn map_api_message(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("incorrect api key") || lower.contains("invalid_api_key") {
        "API key inválida. Revísala en Ajustes.".to_string()
    } else if lower.contains("rate limit") {
        "Límite de velocidad. Espera un momento y vuelve a intentar.".to_string()
    } else if lower.contains("context") && lower.contains("length") {
        "El contexto es demasiado largo. Empieza una conversación nueva.".to_string()
    } else {
        message.to_string()
    }
}
