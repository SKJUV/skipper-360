pub fn get_default_simple_keywords() -> Vec<String> {
    vec![
        "password:".into(),
        "mot de passe:".into(),
        "mot de passe".into(),
        "passphrase".into(),
        "passwd:".into(),
        "authentication password".into(),
    ]
}

pub fn get_default_combined_keywords() -> Vec<Vec<String>> {
    vec![
        vec!["sudo".into(), "password".into()],
        vec!["sudo".into(), "mot de passe".into()],
        vec!["enter".into(), "passphrase".into()],
        vec!["enter".into(), "password".into()],
    ]
}
