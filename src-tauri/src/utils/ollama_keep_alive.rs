//! Parser for Ollama's `OLLAMA_KEEP_ALIVE` environment variable.
//!
//! Ollama accepts:
//! - `5m`, `24h`, `30s` (unit suffix: `s`, `m`, `h`)
//! - plain integer — interpreted as seconds
//! - `-1` — keep loaded forever
//! - `0` — unload immediately
//!
//! We map it to an integer count of minutes for Murmure's STT idle timer.
//! `-1` maps to `u32::MAX` (sentinel for "never"). `0` maps to `1` so the
//! STT model doesn't reload after every single dictation — Ollama's
//! "immediate" semantic is too aggressive for an in-process ORT model.

const NEVER: u32 = u32::MAX;

/// Parse an `OLLAMA_KEEP_ALIVE` value. Returns `None` if the input is
/// empty or unparseable. Returns `Some(u32::MAX)` for `-1`.
pub fn parse(raw: &str) -> Option<u32> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }

    if s == "-1" {
        return Some(NEVER);
    }

    let (num_str, unit) = split_num_unit(s);
    let num: i64 = num_str.parse().ok()?;
    if num < 0 {
        return None;
    }

    let minutes_i64: i64 = match unit {
        "s" | "" => ((num + 59) / 60).max(1),
        "m" => num.max(1),
        "h" => num.saturating_mul(60),
        _ => return None,
    };
    // Clamp to u32::MAX - 1 so absurd inputs never collide with the NEVER
    // sentinel (u32::MAX) and never wrap silently through `as u32`.
    let minutes = u32::try_from(minutes_i64).unwrap_or(NEVER - 1);

    // Ollama's `0` means "unload immediately" (seconds). For an in-process
    // STT model, reloading after every dictation is hostile UX — clamp to 1.
    Some(minutes.max(1))
}

fn split_num_unit(s: &str) -> (&str, &str) {
    let idx = s
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit() && *c != '-')
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    (&s[..idx], &s[idx..])
}

/// Read and parse `OLLAMA_KEEP_ALIVE` from the process env. Cheap —
/// `std::env::var` is a HashMap lookup; callers can invoke this on every
/// idle-timer setup without measurable cost.
pub fn from_env() -> Option<u32> {
    std::env::var("OLLAMA_KEEP_ALIVE").ok().and_then(|v| parse(&v))
}

/// Return `true` iff the parsed value means "never unload".
pub fn is_never(minutes: u32) -> bool {
    minutes == NEVER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minute_suffix() {
        assert_eq!(parse("5m"), Some(5));
        assert_eq!(parse("60m"), Some(60));
    }

    #[test]
    fn parses_hour_suffix() {
        assert_eq!(parse("1h"), Some(60));
        assert_eq!(parse("24h"), Some(1440));
    }

    #[test]
    fn parses_second_suffix_rounds_up() {
        assert_eq!(parse("30s"), Some(1));
        assert_eq!(parse("60s"), Some(1));
        assert_eq!(parse("61s"), Some(2));
        assert_eq!(parse("300s"), Some(5));
    }

    #[test]
    fn plain_integer_is_seconds() {
        assert_eq!(parse("300"), Some(5));
        assert_eq!(parse("0"), Some(1));
    }

    #[test]
    fn minus_one_means_never() {
        assert_eq!(parse("-1"), Some(NEVER));
        assert!(is_never(parse("-1").unwrap()));
    }

    #[test]
    fn zero_clamps_to_one_minute() {
        assert_eq!(parse("0"), Some(1));
        assert_eq!(parse("0s"), Some(1));
        assert_eq!(parse("0m"), Some(1));
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(parse("  5m  "), Some(5));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("abc"), None);
        assert_eq!(parse("5x"), None);
        assert_eq!(parse("-5m"), None);
    }

    #[test]
    fn absurd_hour_value_clamps_below_never_sentinel() {
        // Without the clamp, `as u32` would wrap and could land on
        // u32::MAX (the NEVER sentinel), confusing `is_never`.
        let parsed = parse("9999999999h").unwrap();
        assert!(parsed < NEVER);
        assert!(!is_never(parsed));
    }
}
