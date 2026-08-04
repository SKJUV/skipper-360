use skipper_core::{MatchMode, WhitelistEntry};

#[test]
fn test_whitelist_entry_matching() {
    let exact_entry = WhitelistEntry {
        command: "sudo pacman -Syu".to_string(),
        keyring_key: "whitelist:sudo_pacman_-Syu".to_string(),
        match_mode: MatchMode::Exact,
    };

    let prefix_entry = WhitelistEntry {
        command: "ssh user@serveur-prod".to_string(),
        keyring_key: "whitelist:ssh_user_prod".to_string(),
        match_mode: MatchMode::Prefix,
    };

    assert_eq!(exact_entry.match_mode, MatchMode::Exact);
    assert_eq!(prefix_entry.match_mode, MatchMode::Prefix);

    let full_cmd = "ssh user@serveur-prod -p 2222";
    assert!(full_cmd.starts_with(&prefix_entry.command));
}
