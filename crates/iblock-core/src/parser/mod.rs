//! Zero-copy streaming parser for Adblock Plus & uBlock Origin static filter lists.

use crate::filter::{FilterAction, NetworkRule, PatternKind, RequestTypes};

#[derive(Debug, PartialEq, Eq)]
pub enum ParsedLine {
    EmptyOrComment,
    Network(NetworkRule),
    Cosmetic(String),
}

/// Parses a single line from a filter list into a strongly-typed rule.
pub fn parse_line(line: &str) -> ParsedLine {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('[') {
        return ParsedLine::EmptyOrComment;
    }

    // Check for cosmetic filters: `##`, `#@#`, `#?#`
    if trimmed.contains("##") || trimmed.contains("#@#") || trimmed.contains("#?#") {
        return ParsedLine::Cosmetic(trimmed.to_string());
    }

    // Parse network rule
    if let Some(rule) = parse_network_rule(trimmed) {
        ParsedLine::Network(rule)
    } else {
        ParsedLine::EmptyOrComment
    }
}

/// Parses a network rule line (e.g., `||example.com^$script,third-party`).
fn parse_network_rule(line: &str) -> Option<NetworkRule> {
    let mut s = line;

    // Check exception prefix `@@`
    let is_exception = if s.starts_with("@@") {
        s = &s[2..];
        true
    } else {
        false
    };

    // Separate pattern and modifiers by `$`
    let (pattern_str, modifiers_opt) = match s.rfind('$') {
        Some(idx) => (&s[..idx], Some(&s[idx + 1..])),
        None => (s, None),
    };

    if pattern_str.is_empty() {
        return None;
    }

    // Parse pattern kind
    let pattern = if pattern_str.starts_with("||") {
        let domain_part = &pattern_str[2..];
        if let Some(slash_idx) = domain_part.find('/') {
            let domain = domain_part[..slash_idx].trim_end_matches('^').to_string();
            let path = domain_part[slash_idx..].trim_end_matches('^').to_string();
            PatternKind::DomainAnchor { domain, path }
        } else {
            let domain = domain_part.trim_end_matches('^').to_string();
            PatternKind::DomainAnchor { domain, path: String::new() }
        }
    } else if pattern_str.starts_with('|') && pattern_str.ends_with('|') && pattern_str.len() > 1 {
        PatternKind::Exact(pattern_str[1..pattern_str.len() - 1].to_string())
    } else if pattern_str.starts_with('|') {
        PatternKind::LeftAnchor(pattern_str[1..].to_string())
    } else if pattern_str.ends_with('|') {
        PatternKind::RightAnchor(pattern_str[..pattern_str.len() - 1].to_string())
    } else if pattern_str.starts_with('/') && pattern_str.ends_with('/') && pattern_str.len() > 1 {
        PatternKind::Regex(pattern_str[1..pattern_str.len() - 1].to_string())
    } else {
        PatternKind::Substring(pattern_str.to_string())
    };

    let mut included_types = RequestTypes::empty();
    let mut excluded_types = RequestTypes::empty();
    let mut included_domains = Vec::new();
    let mut excluded_domains = Vec::new();
    let mut action = if is_exception {
        FilterAction::Allow
    } else {
        FilterAction::Block
    };

    // Parse modifiers if present
    if let Some(modifiers) = modifiers_opt {
        for opt in modifiers.split(',') {
            let opt = opt.trim();
            if opt.is_empty() {
                continue;
            }

            if let Some(redirect_val) = opt.strip_prefix("redirect=") {
                if !is_exception {
                    action = FilterAction::Redirect(redirect_val.to_string());
                }
            } else if let Some(domains) = opt.strip_prefix("domain=") {
                for d in domains.split('|') {
                    if let Some(excluded) = d.strip_prefix('~') {
                        if !excluded.is_empty() {
                            excluded_domains.push(excluded.to_string());
                        }
                    } else if !d.is_empty() {
                        included_domains.push(d.to_string());
                    }
                }
            } else if let Some(param) = opt.strip_prefix("removeparam=") {
                if !is_exception {
                    action = FilterAction::RemoveParam(param.to_string());
                }
            } else {
                let (negated, token) = if let Some(rest) = opt.strip_prefix('~') {
                    (true, rest)
                } else {
                    (false, opt)
                };

                let flag = match token {
                    "script" => RequestTypes::SCRIPT,
                    "image" => RequestTypes::IMAGE,
                    "stylesheet" | "css" => RequestTypes::STYLESHEET,
                    "xmlhttprequest" | "xhr" => RequestTypes::XMLHTTPREQUEST,
                    "subdocument" | "frame" => RequestTypes::SUBDOCUMENT,
                    "websocket" => RequestTypes::WEBSOCKET,
                    "media" => RequestTypes::MEDIA,
                    "font" => RequestTypes::FONT,
                    "other" => RequestTypes::OTHER,
                    "1p" | "first-party" => RequestTypes::FIRST_PARTY,
                    "3p" | "third-party" => RequestTypes::THIRD_PARTY,
                    "important" => RequestTypes::IMPORTANT,
                    "match-case" => RequestTypes::MATCH_CASE,
                    _ => RequestTypes::empty(),
                };

                if negated {
                    excluded_types |= flag;
                } else {
                    included_types |= flag;
                }
            }
        }
    }

    Some(NetworkRule {
        pattern,
        action,
        included_types,
        excluded_types,
        included_domains,
        excluded_domains,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_comments() {
        assert_eq!(parse_line("! Title: EasyList"), ParsedLine::EmptyOrComment);
        assert_eq!(parse_line("[Adblock Plus 2.0]"), ParsedLine::EmptyOrComment);
        assert_eq!(parse_line("   "), ParsedLine::EmptyOrComment);
    }

    #[test]
    fn test_parse_domain_anchor() {
        let line = "||doubleclick.net^$third-party,script";
        let parsed = parse_line(line);
        if let ParsedLine::Network(rule) = parsed {
            assert_eq!(
                rule.pattern,
                PatternKind::DomainAnchor {
                    domain: "doubleclick.net".to_string(),
                    path: String::new()
                }
            );
            assert_eq!(rule.action, FilterAction::Block);
            assert!(rule.included_types.contains(RequestTypes::THIRD_PARTY));
            assert!(rule.included_types.contains(RequestTypes::SCRIPT));
        } else {
            panic!("Expected Network rule");
        }
    }

    #[test]
    fn test_parse_exception_rule() {
        let line = "@@||analytics.google.com^$domain=example.com";
        let parsed = parse_line(line);
        if let ParsedLine::Network(rule) = parsed {
            assert!(rule.is_exception());
            assert_eq!(
                rule.pattern,
                PatternKind::DomainAnchor {
                    domain: "analytics.google.com".to_string(),
                    path: String::new()
                }
            );
            assert_eq!(rule.included_domains, vec!["example.com"]);
        } else {
            panic!("Expected Network rule");
        }
    }

    #[test]
    fn test_parse_redirect_rule() {
        let line = "||google-analytics.com/analytics.js$script,redirect=google-analytics_analytics.js";
        let parsed = parse_line(line);
        if let ParsedLine::Network(rule) = parsed {
            assert_eq!(
                rule.action,
                FilterAction::Redirect("google-analytics_analytics.js".to_string())
            );
        } else {
            panic!("Expected Network rule");
        }
    }

    #[test]
    fn test_parse_cosmetic_rule() {
        let line = "example.com##.ad-banner";
        let parsed = parse_line(line);
        assert!(matches!(parsed, ParsedLine::Cosmetic(_)));
    }
}
