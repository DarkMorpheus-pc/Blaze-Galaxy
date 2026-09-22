use anyhow::{bail, Context, Result};
use std::path::PathBuf;

fn get_noctalia_settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
    PathBuf::from(home).join(".local/state/noctalia/settings.toml")
}

pub fn get_config_json() -> Result<String> {
    let path = get_noctalia_settings_path();
    if !path.exists() {
        return Ok("{}".to_string());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    let toml_val: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML from {:?}", path))?;
    let json_val = serde_json::to_string(&toml_val)?;
    Ok(json_val)
}

pub fn set_config_value(key_path: &str, raw_value: &str) -> Result<()> {
    let path = get_noctalia_settings_path();
    let mut toml_val: toml::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        toml::from_str(&content)?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    let keys: Vec<&str> = key_path.split('.').collect();
    if keys.is_empty() {
        bail!("Empty key path");
    }

    let parsed_val: toml::Value = if raw_value.eq_ignore_ascii_case("true") {
        toml::Value::Boolean(true)
    } else if raw_value.eq_ignore_ascii_case("false") {
        toml::Value::Boolean(false)
    } else if let Ok(i) = raw_value.parse::<i64>() {
        toml::Value::Integer(i)
    } else if let Ok(f) = raw_value.parse::<f64>() {
        toml::Value::Float(f)
    } else {
        toml::Value::String(raw_value.to_string())
    };

    let mut current = &mut toml_val;
    for k in &keys[..keys.len() - 1] {
        if !current.is_table() {
            *current = toml::Value::Table(toml::map::Map::new());
        }
        let table = current.as_table_mut().unwrap();
        if !table.contains_key(*k) {
            table.insert(k.to_string(), toml::Value::Table(toml::map::Map::new()));
        }
        current = table.get_mut(*k).unwrap();
    }

    if let Some(table) = current.as_table_mut() {
        table.insert(keys.last().unwrap().to_string(), parsed_val);
    }

    let toml_str = toml::to_string_pretty(&toml_val)?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, toml_str)?;
    Ok(())
}

pub fn sync_taskbar_state(enabled: bool) -> Result<()> {
    let path = get_noctalia_settings_path();
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&path)?;
    let mut toml_val: toml::Value = toml::from_str(&content)?;

    if let Some(table) = toml_val.as_table_mut() {
        if let Some(bar_val) = table.get_mut("bar").and_then(|v| v.as_table_mut()) {
            let order_vec = if enabled {
                vec![
                    toml::Value::String("default".to_string()),
                    toml::Value::String("taskbar".to_string()),
                ]
            } else {
                vec![toml::Value::String("default".to_string())]
            };
            bar_val.insert("order".to_string(), toml::Value::Array(order_vec));
        }
    }

    let toml_str = toml::to_string_pretty(&toml_val)?;
    std::fs::write(&path, toml_str)?;

    let _ = std::process::Command::new("noctalia")
        .args(["msg", "reload"])
        .status();

    Ok(())
}
