/// Approximate token counting for LLM context window management.
/// Uses a heuristic of ~4 characters per token, which closely matches
/// GPT/Claude tokenization for source code.

pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    // Heuristic: ~4 chars per token for code (matches tiktoken o200k_base closely)
    // More accurate than word count for code with lots of punctuation/symbols
    (text.len() + 3) / 4
}

pub fn format_token_count(tokens: usize) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}k", tokens as f64 / 1_000.0)
    } else {
        format!("{}", tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens_empty() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn test_count_tokens_exact_boundaries() {
        assert_eq!(count_tokens("a"), 1);       // 1 char → (1+3)/4 = 1
        assert_eq!(count_tokens("ab"), 1);      // 2 chars → (2+3)/4 = 1
        assert_eq!(count_tokens("abc"), 1);     // 3 chars → (3+3)/4 = 1
        assert_eq!(count_tokens("abcd"), 1);    // 4 chars → (4+3)/4 = 1
        assert_eq!(count_tokens("abcde"), 2);   // 5 chars → (5+3)/4 = 2
        assert_eq!(count_tokens("abcdefgh"), 2); // 8 chars → (8+3)/4 = 2
        assert_eq!(count_tokens("abcdefghi"), 3); // 9 chars → (9+3)/4 = 3
    }

    #[test]
    fn test_count_tokens_code() {
        assert_eq!(count_tokens("fn main() {}"), 3); // 12 chars / 4
    }

    #[test]
    fn test_count_tokens_whitespace() {
        assert!(count_tokens("    ") > 0);
        assert!(count_tokens("\n\n\n") > 0);
    }

    #[test]
    fn test_format_token_count_below_thousand() {
        assert_eq!(format_token_count(0), "0");
        assert_eq!(format_token_count(1), "1");
        assert_eq!(format_token_count(500), "500");
        assert_eq!(format_token_count(999), "999");
    }

    #[test]
    fn test_format_token_count_thousands() {
        assert_eq!(format_token_count(1000), "1.0k");
        assert_eq!(format_token_count(1500), "1.5k");
        assert_eq!(format_token_count(10_000), "10.0k");
        assert_eq!(format_token_count(999_999), "1000.0k");
    }

    #[test]
    fn test_format_token_count_millions() {
        assert_eq!(format_token_count(1_000_000), "1.0M");
        assert_eq!(format_token_count(1_500_000), "1.5M");
        assert_eq!(format_token_count(10_000_000), "10.0M");
    }
}
