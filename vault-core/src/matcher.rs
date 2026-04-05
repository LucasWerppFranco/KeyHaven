//! Search and matching utilities for vault entries.

/// Check if a query matches an entry title or tags
pub fn matches(query: &str, title: &str, _tags: Option<&str>) -> bool {
    let query_lower = query.to_lowercase();
    let title_lower = title.to_lowercase();

    // Simple substring match (can be enhanced with fuzzy matching)
    title_lower.contains(&query_lower)
}

/// Calculate relevance score for sorting search results
pub fn relevance_score(query: &str, title: &str) -> usize {
    let query_lower = query.to_lowercase();
    let title_lower = title.to_lowercase();

    if title_lower == query_lower {
        return 100; // Exact match
    }

    if title_lower.starts_with(&query_lower) {
        return 50; // Starts with query
    }

    if title_lower.contains(&query_lower) {
        return 25; // Contains query
    }

    0 // No match
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(matches("github", "github", None));
        assert_eq!(relevance_score("github", "github"), 100);
    }

    #[test]
    fn test_case_insensitive_match() {
        assert!(matches("GITHUB", "GitHub", None));
        assert!(matches("github", "GitHub", None));
    }

    #[test]
    fn test_partial_match() {
        assert!(matches("git", "github", None));
        assert_eq!(relevance_score("git", "github"), 50);
    }

    #[test]
    fn test_no_match() {
        assert!(!matches("twitter", "github", None));
        assert_eq!(relevance_score("twitter", "github"), 0);
    }
}
