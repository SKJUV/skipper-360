use regex::RegexSet;
use skipper_core::patterns::DEFAULT_PATTERNS;

pub struct PromptDetector {
    regex_set: RegexSet,
    patterns: Vec<String>,
}

impl PromptDetector {
    pub fn new(custom_patterns: &[String]) -> Self {
        let mut patterns: Vec<String> = DEFAULT_PATTERNS.iter().map(|s| s.to_string()).collect();
        patterns.extend(custom_patterns.iter().cloned());

        let regex_set =
            RegexSet::new(&patterns).expect("Failed to compile prompt detection RegexSet");

        Self {
            regex_set,
            patterns,
        }
    }

    pub fn detect(&self, text: &str) -> Option<String> {
        let matches = self.regex_set.matches(text);
        if let Some(index) = matches.into_iter().next() {
            return self.patterns.get(index).cloned();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_detection() {
        let detector = PromptDetector::new(&[]);
        assert!(detector.detect("[sudo] password for skjuve:").is_some());
        assert!(detector.detect("Mot de passe :").is_some());
        assert!(detector.detect("user@server's password:").is_some());
        assert!(detector
            .detect("Enter passphrase for key /home/user/.ssh/id_rsa:")
            .is_some());
        assert!(detector.detect("Hello world output line").is_none());
    }
}
