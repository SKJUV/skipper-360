use skipper_core::Config;

#[test]
fn test_focus_configuration_default() {
    let config = Config::default();
    assert!(config.general.focus_detection);
}
