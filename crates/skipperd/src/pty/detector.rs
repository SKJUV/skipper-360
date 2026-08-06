use skipper_core::KeywordsConfig;

pub struct PromptDetector {
    simple_keywords: Vec<String>,
    combined_keywords: Vec<Vec<String>>,
}

impl PromptDetector {
    pub fn new(keywords_config: &KeywordsConfig) -> Self {
        Self {
            simple_keywords: keywords_config.simple_keywords.clone(),
            combined_keywords: keywords_config.combined_keywords.clone(),
        }
    }

    pub fn detect(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        // 1. Recherche par mots-clés simples (contient le mot-clé)
        for kw in &self.simple_keywords {
            let kw_lower = kw.to_lowercase();
            if text_lower.contains(&kw_lower) {
                return Some(format!("Mot-clé détecté : '{}'", kw));
            }
        }

        // 2. Recherche par combinaisons de mots-clés (contient TOUS les mots-clés)
        for combo in &self.combined_keywords {
            if !combo.is_empty()
                && combo
                    .iter()
                    .all(|kw| text_lower.contains(&kw.to_lowercase()))
            {
                return Some(format!("Combinaison de mots-clés détectée : {:?}", combo));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_prompt_detection() {
        let keywords_config = KeywordsConfig::default();
        let detector = PromptDetector::new(&keywords_config);

        assert!(detector.detect("[sudo] password for skjuve:").is_some());
        assert!(detector.detect("Mot de passe :").is_some());
        assert!(detector.detect("user@server's password:").is_some());
        assert!(detector
            .detect("Enter passphrase for key /home/user/.ssh/id_rsa:")
            .is_some());
        assert!(detector.detect("Hello world output line").is_none());
    }
}
