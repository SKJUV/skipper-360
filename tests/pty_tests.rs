use skipper_core::{Config, MatchMode};

#[test]
fn test_pty_pattern_and_config_resolution() {
    let mut config = Config::default();
    assert_eq!(config.general.timeout_seconds, 10);
    assert!(config.general.focus_detection);

    let key = config.add_whitelist_entry("sudo pacman -Syu", MatchMode::Exact);
    assert_eq!(config.whitelist.entries.len(), 1);
    assert_eq!(config.whitelist.entries[0].keyring_key, key);
}
