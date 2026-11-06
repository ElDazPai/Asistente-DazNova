//! Tarifas aproximadas en USD por millón de tokens.
//!
//! Sirven para telemetría de gasto, no para facturación. Los modelos locales
//! se registran a coste cero.

struct Rates {
    input: f64,
    output: f64,
}

fn rates_for(model: &str) -> Rates {
    let name = model.to_ascii_lowercase();

    if name.contains("llama")
        || name.contains("mistral")
        || name.contains("qwen")
        || name.contains("phi")
        || name.contains("gemma")
        || name.contains("deepseek") && name.contains("local")
    {
        return Rates {
            input: 0.0,
            output: 0.0,
        };
    }

    if name.contains("gpt-4.1-nano") {
        return Rates {
            input: 0.10,
            output: 0.40,
        };
    }
    if name.contains("gpt-4.1-mini") {
        return Rates {
            input: 0.40,
            output: 1.60,
        };
    }
    if name.contains("gpt-4.1") {
        return Rates {
            input: 2.00,
            output: 8.00,
        };
    }
    if name.contains("gpt-4o-mini") {
        return Rates {
            input: 0.15,
            output: 0.60,
        };
    }
    if name.contains("gpt-4o") {
        return Rates {
            input: 2.50,
            output: 10.00,
        };
    }
    if name.contains("o3-mini") {
        return Rates {
            input: 1.10,
            output: 4.40,
        };
    }

    Rates {
        input: 0.50,
        output: 1.50,
    }
}

/// Estima el coste en dólares a partir de tokens de entrada y salida.
pub fn estimate(model: &str, tokens_in: i64, tokens_out: i64) -> f64 {
    let rates = rates_for(model);
    let cost = (tokens_in as f64 / 1_000_000.0) * rates.input
        + (tokens_out as f64 / 1_000_000.0) * rates.output;
    (cost * 1_000_000.0).round() / 1_000_000.0
}

/// Aproximación tosca cuando el proveedor no envía `usage`.
pub fn estimate_tokens(text: &str) -> i64 {
    let chars = text.chars().count().max(1);
    ((chars as f64) / 4.0).ceil() as i64
}
