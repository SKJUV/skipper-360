use skipper_core::{MatchMode, WhitelistEntry};

#[test]
fn test_whitelist_entry_matching() {
    let exact_entry = WhitelistEntry {
        command: "sudo pacman -Syu".to_string(),
        keyring_key: "whitelist:sudo_pacman_-Syu".to_string(),
        match_mode: MatchMode::Exact,
        expires_at: None,
    };

    let prefix_entry = WhitelistEntry {
        command: "ssh user@serveur-prod".to_string(),
        keyring_key: "whitelist:ssh_user_prod".to_string(),
        match_mode: MatchMode::Prefix,
        expires_at: None,
    };

    assert_eq!(exact_entry.match_mode, MatchMode::Exact);
    assert_eq!(prefix_entry.match_mode, MatchMode::Prefix);

    let full_cmd = "ssh user@serveur-prod -p 2222";
    assert!(full_cmd.starts_with(&prefix_entry.command));
}

#[test]
fn test_auto_sync_shims_on_whitelist_mutation() {
    use skipper_core::{sync_shims, Config, MatchMode};

    let mut cfg = Config::default();
    cfg.add_whitelist_entry("ssh user@host", MatchMode::Prefix);
    cfg.add_whitelist_entry("pacman -Syu", MatchMode::Exact);

    let synced = sync_shims(&cfg).expect("Sync shims failed");
    assert!(synced.contains(&"ssh".to_string()));
    assert!(synced.contains(&"pacman".to_string()));

    cfg.remove_whitelist_entry("ssh user@host");
    let synced_after = sync_shims(&cfg).expect("Sync shims failed");
    assert!(!synced_after.contains(&"ssh".to_string()));
    assert!(synced_after.contains(&"pacman".to_string()));
}
