use crate::settings::AppSettings;
use tauri::{command, AppHandle};

#[command]
pub fn get_all_settings(app: AppHandle) -> Result<AppSettings, String> {
    Ok(crate::settings::load_settings(&app))
}

#[command]
pub fn get_current_language(app: AppHandle) -> Result<String, String> {
    let s = crate::settings::load_settings(&app);
    Ok(s.language)
}

#[command]
pub fn set_current_language(app: AppHandle, lang: String) -> Result<(), String> {
    const SUPPORTED_LANGUAGES: &[&str] = &["default", "en", "fr"];

    if !SUPPORTED_LANGUAGES.contains(&lang.as_str()) {
        return Err(format!("Unsupported language code: {}", lang));
    }

    let mut s = crate::settings::load_settings(&app);
    s.language = lang;
    crate::settings::save_settings(&app, &s)
}

#[command]
pub fn get_current_mic_id(app: AppHandle) -> Result<Option<String>, String> {
    let s = crate::settings::load_settings(&app);
    Ok(s.mic_id)
}

#[command]
pub fn set_current_mic_id(
    app: AppHandle,
    mic_id: Option<String>,
    mic_label: Option<String>,
) -> Result<(), String> {
    let mut s = crate::settings::load_settings(&app);
    s.mic_id = mic_id.clone();
    s.mic_label = mic_label;
    crate::settings::save_settings(&app, &s)?;
    crate::audio::microphone::update_mic_cache(&app, mic_id);
    Ok(())
}

#[command]
pub fn get_current_mic_label(app: AppHandle) -> Result<Option<String>, String> {
    let s = crate::settings::load_settings(&app);
    Ok(s.mic_label)
}

#[command]
pub fn get_mic_list() -> Result<Vec<crate::audio::types::MicInfo>, String> {
    let mic_list = crate::audio::microphone::get_mic_list();
    Ok(mic_list)
}

#[command]
pub fn set_sound_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut s = crate::settings::load_settings(&app);
    s.sound_enabled = enabled;
    crate::settings::save_settings(&app, &s)
}

#[command]
pub fn set_log_level(app: AppHandle, level: String) -> Result<(), String> {
    let valid_levels = ["off", "error", "warn", "info", "debug", "trace"];
    if !valid_levels.contains(&level.to_lowercase().as_str()) {
        return Err(format!("Invalid log level: {}", level));
    }

    let mut s = crate::settings::load_settings(&app);
    s.log_level = level.clone();
    crate::settings::save_settings(&app, &s)?;

    if let Ok(level_filter) = std::str::FromStr::from_str(&level) {
        log::set_max_level(level_filter);
    }

    Ok(())
}

#[command]
pub fn set_show_in_dock(app: AppHandle, show: bool) -> Result<(), String> {
    let mut s = crate::settings::load_settings(&app);
    s.show_in_dock = show;
    crate::settings::save_settings(&app, &s)
}

#[command]
pub fn get_idle_unload_minutes(app: AppHandle) -> Result<u32, String> {
    let s = crate::settings::load_settings(&app);
    Ok(s.idle_unload_minutes)
}

#[command]
pub fn set_idle_unload_minutes(app: AppHandle, minutes: u32) -> Result<(), String> {
    const ALLOWED: &[u32] = &[0, 5, 15, 30, 60];
    if !ALLOWED.contains(&minutes) {
        return Err(format!("Invalid idle unload preset: {}", minutes));
    }

    let mut s = crate::settings::load_settings(&app);
    s.idle_unload_minutes = minutes;
    crate::settings::save_settings(&app, &s)?;

    // Apply immediately so "after N min" counts from save, not next dictation.
    if minutes == 0 {
        crate::audio::cancel_pending_idle_unload(&app);
    } else {
        crate::audio::schedule_idle_unload(&app);
    }

    Ok(())
}

/// Snapshot of `OLLAMA_KEEP_ALIVE` for the UI opt-in checkbox.
#[derive(serde::Serialize)]
pub struct OllamaKeepAliveInfo {
    /// Raw env var value (if present). `None` when unset.
    pub raw: Option<String>,
    /// Parsed minutes (if the raw value is parseable). `None` for unparseable.
    pub minutes: Option<u32>,
    /// `true` when the parsed value is the "never" sentinel (`-1`).
    pub never: bool,
    /// `true` when LLM Connect has at least one Local-provider mode —
    /// only condition under which Ollama's env var is meaningful for
    /// Murmure's STT.
    pub llm_local_active: bool,
}

#[command]
pub fn detect_ollama_keep_alive(app: AppHandle) -> OllamaKeepAliveInfo {
    let raw = std::env::var("OLLAMA_KEEP_ALIVE").ok();
    let minutes = raw.as_deref().and_then(crate::utils::ollama_keep_alive::parse);
    let never = minutes
        .map(crate::utils::ollama_keep_alive::is_never)
        .unwrap_or(false);

    let llm = crate::llm::helpers::load_llm_connect_settings(&app);
    let llm_local_active = llm
        .modes
        .iter()
        .any(|m| m.provider == crate::llm::types::LLMProvider::Local);

    OllamaKeepAliveInfo {
        raw,
        // Expose `None` for the "never" sentinel so the UI doesn't show
        // `4294967295 min` — the `never` boolean carries that meaning.
        minutes: minutes.filter(|_| !never),
        never,
        llm_local_active,
    }
}

#[command]
pub fn get_idle_unload_follow_ollama(app: AppHandle) -> Result<bool, String> {
    let s = crate::settings::load_settings(&app);
    Ok(s.idle_unload_follow_ollama)
}

#[command]
pub fn set_idle_unload_follow_ollama(app: AppHandle, follow: bool) -> Result<(), String> {
    let mut s = crate::settings::load_settings(&app);
    s.idle_unload_follow_ollama = follow;
    crate::settings::save_settings(&app, &s)?;

    // Re-arm the timer so the new policy takes effect immediately.
    crate::audio::cancel_pending_idle_unload(&app);
    crate::audio::schedule_idle_unload(&app);

    Ok(())
}
