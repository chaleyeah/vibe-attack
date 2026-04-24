use strsim::levenshtein;

pub struct PhraseMatcher {
    threshold: f32,
}

impl PhraseMatcher {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    pub fn normalize(input: &str) -> String {
        input
            .to_lowercase()
            .trim()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn find_best_match<'a>(
        &self,
        input: &str,
        candidates: impl Iterator<Item = (&'a str, &'a str)>,
    ) -> Option<(&'a str, f32)> {
        let norm_input = Self::normalize(input);
        if norm_input.is_empty() {
            return None;
        }

        let mut best_match = None;
        let mut best_score = 0.0;

        for (id, phrase) in candidates {
            let norm_phrase = Self::normalize(phrase);
            if norm_phrase.is_empty() {
                continue;
            }

            let max_len = norm_input.chars().count().max(norm_phrase.chars().count()) as f32;
            let dist = levenshtein(&norm_input, &norm_phrase) as f32;
            let score = if max_len > 0.0 {
                1.0 - (dist / max_len)
            } else {
                0.0
            };

            if score >= self.threshold && score > best_score {
                best_score = score;
                best_match = Some((id, score));
            }
        }

        best_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(PhraseMatcher::normalize("Hello, World!"), "hello world");
        assert_eq!(PhraseMatcher::normalize("  eagle   airstrike...  "), "eagle airstrike");
    }

    #[test]
    fn test_exact_match() {
        let matcher = PhraseMatcher::new(0.8);
        let candidates = vec![("macro1", "eagle airstrike")];
        let matched = matcher.find_best_match("eagle airstrike", candidates.into_iter());
        assert_eq!(matched.unwrap().0, "macro1");
        assert_eq!(matched.unwrap().1, 1.0);
    }

    #[test]
    fn test_fuzzy_match() {
        let matcher = PhraseMatcher::new(0.8);
        let candidates = vec![("macro1", "eagle airstrike")];
        // "eagal airstrike" has levenshtein distance of 1 from "eagle airstrike"
        let matched = matcher.find_best_match("eagal airstrike", candidates.into_iter());
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().0, "macro1");
    }

    #[test]
    fn test_threshold_rejection() {
        let matcher = PhraseMatcher::new(0.8);
        let candidates = vec![("macro1", "eagle airstrike")];
        let matched = matcher.find_best_match("orbital strike", candidates.into_iter());
        assert!(matched.is_none());
    }

    #[test]
    fn test_punctuation_stripping() {
        let matcher = PhraseMatcher::new(0.8);
        let candidates = vec![("macro1", "eagle airstrike")];
        let matched = matcher.find_best_match("eagle airstrike!!!", candidates.into_iter());
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().0, "macro1");
    }
}
