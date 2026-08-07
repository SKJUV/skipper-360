use skipper_core::{Config, MatchMode};

#[test]
fn test_whitelist_matching_rules() {
    let mut config = Config::default();
    config.add_whitelist_entry("ssh user@srv", MatchMode::Prefix);

    let matched = config.whitelist.entries.iter().any(|e| {
        match e.match_mode {
            MatchMode::Exact => "ssh user@srv" == e.command,
            MatchMode::Prefix => "ssh user@srv node1".starts_with(&e.command),
        }
    });

    assert!(matched);
}
