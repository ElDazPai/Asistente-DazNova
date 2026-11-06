//! Acceso al llavero del sistema operativo.
//!
//! SQLite solo guarda referencias (`token_ref`). Las claves viven aquí.

const SERVICE: &str = "com.daznova.asistente";

fn map_error(error: keyring::Error) -> String {
    match error {
        keyring::Error::NoEntry => "no hay secreto guardado".to_string(),
        other => format!("llavero: {other}"),
    }
}

pub fn set_secret(reference: &str, value: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, reference).map_err(map_error)?;
    entry.set_password(value).map_err(map_error)
}

pub fn get_secret(reference: &str) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(SERVICE, reference).map_err(map_error)?;
    match entry.get_password() {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(map_error(error)),
    }
}

pub fn delete_secret(reference: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, reference).map_err(map_error)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(map_error(error)),
    }
}

pub fn has_secret(reference: &str) -> Result<bool, String> {
    Ok(get_secret(reference)?.is_some())
}
