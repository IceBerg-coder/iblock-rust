//! Static Network Filtering Engine (SNFE).
//!
//! Orchestrates the `HNTrie`, `BidiTrie`, and rule buckets to evaluate network requests
//! against thousands of filter rules in sub-microsecond time.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::cosmetic::{CosmeticEngine, CosmeticResult};
use crate::dynamic::{DynamicAction, DynamicFirewall, DynamicType};
use crate::filter::{FilterAction, NetworkRule, PatternKind, RequestTypes};
use crate::parser::{parse_line, ParsedLine};
use crate::trie::HNTrie;

/// Evaluation outcome for an HTTP network request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchResult {
    /// Request explicitly allowed by an exception rule (whitelist)
    Allow,
    /// Request blocked by a filter rule
    Block,
    /// Request redirected to a local neutral surrogate shim
    Redirect(String),
    /// No matching rule found
    NoMatch,
}

/// Request context used for evaluation.
#[derive(Clone, Copy, Debug)]
pub struct RequestContext<'a> {
    pub url: &'a str,
    pub hostname: &'a str,
    pub source_domain: &'a str,
    pub request_type: RequestTypes,
    pub is_third_party: bool,
}

/// The core static network filtering engine.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StaticNetFilteringEngine {
    /// Fast domain-anchor blacklist trie: `||domain.com^`
    domain_blacklist_trie: HNTrie,
    /// Fast domain-anchor whitelist/exception trie: `@@||domain.com^`
    domain_whitelist_trie: HNTrie,
    /// General network rules with complex modifiers or URL patterns
    general_rules: Vec<NetworkRule>,
    /// Exception / allow rules with complex constraints
    exception_rules: Vec<NetworkRule>,
    /// Cosmetic, procedural, and scriptlet injection engine
    pub cosmetic_engine: CosmeticEngine,
    /// Dynamic matrix firewall
    pub dynamic_firewall: DynamicFirewall,
    /// Total count of parsed and loaded rules
    rule_count: usize,
}

impl StaticNetFilteringEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads filter rules from a raw text list (e.g. EasyList, uBlock filters).
    pub fn load_list(&mut self, content: &str) -> usize {
        let mut loaded = 0;
        for line in content.lines() {
            match parse_line(line) {
                ParsedLine::Network(rule) => {
                    self.add_rule(rule);
                    loaded += 1;
                }
                ParsedLine::Cosmetic(_) => {
                    if self.cosmetic_engine.add_line(line) {
                        self.rule_count += 1;
                        loaded += 1;
                    }
                }
                ParsedLine::EmptyOrComment => {}
            }
        }
        loaded
    }

    /// Adds a single parsed network rule into the engine.
    pub fn add_rule(&mut self, rule: NetworkRule) {
        self.rule_count += 1;

        // Optimization: Pure domain anchors with default modifiers go directly into HNTrie
        let is_pure_domain = rule.included_domains.is_empty()
            && rule.excluded_domains.is_empty()
            && rule.included_types.is_empty()
            && rule.excluded_types.is_empty();

        if is_pure_domain {
            if let PatternKind::DomainAnchor { ref domain, ref path } = rule.pattern {
                if path.is_empty() {
                    if rule.is_exception() {
                        self.domain_whitelist_trie.add(domain);
                        return;
                    } else if rule.action == FilterAction::Block {
                        self.domain_blacklist_trie.add(domain);
                        return;
                    }
                }
            }
        }

        if rule.is_exception() {
            self.exception_rules.push(rule);
        } else {
            self.general_rules.push(rule);
        }
    }

    /// Extracts the hostname portion from an HTTP/HTTPS URL.
    pub fn extract_hostname(url: &str) -> &str {
        let without_proto = if let Some(rest) = url.strip_prefix("https://") {
            rest
        } else if let Some(rest) = url.strip_prefix("http://") {
            rest
        } else if let Some(rest) = url.strip_prefix("//") {
            rest
        } else {
            url
        };

        let host_part = without_proto.split(&['/', '?', ':', '#'][..]).next().unwrap_or(without_proto);
        host_part
    }

    /// Determines whether the request is 3rd-party relative to the source/context domain.
    pub fn is_third_party(hostname: &str, source_domain: &str) -> bool {
        if source_domain.is_empty() || hostname.is_empty() {
            return false;
        }
        !hostname.ends_with(source_domain)
    }

    /// Evaluates an incoming network request.
    pub fn match_request(&self, url: &str, source_domain: &str, request_type: RequestTypes) -> MatchResult {
        let hostname = Self::extract_hostname(url);
        let is_third_party = Self::is_third_party(hostname, source_domain);

        let ctx = RequestContext {
            url,
            hostname,
            source_domain,
            request_type,
            is_third_party,
        };

        // 0. Dynamic Matrix Firewall check (highest priority)
        let dynamic_type = if request_type.contains(RequestTypes::SCRIPT) {
            if is_third_party {
                DynamicType::ThirdPartyScript
            } else {
                DynamicType::FirstPartyScript
            }
        } else if request_type.contains(RequestTypes::SUBDOCUMENT) && is_third_party {
            DynamicType::ThirdPartyFrame
        } else if request_type.contains(RequestTypes::IMAGE) {
            DynamicType::Image
        } else if is_third_party {
            DynamicType::ThirdParty
        } else {
            DynamicType::Any
        };

        if let Some(action) = self.dynamic_firewall.evaluate(source_domain, hostname, dynamic_type) {
            match action {
                DynamicAction::Block => return MatchResult::Block,
                DynamicAction::Allow => return MatchResult::Allow,
                DynamicAction::Noop => {} // Fall through to static filter lists
            }
        }

        // 1. Check fast pure-domain whitelist
        if self.domain_whitelist_trie.contains(hostname) {
            return MatchResult::Allow;
        }

        // 2. Check general exception / whitelist rules
        for ex in &self.exception_rules {
            if self.rule_matches(ex, &ctx) {
                return MatchResult::Allow;
            }
        }

        // 3. Check fast pure-domain blacklist
        if self.domain_blacklist_trie.contains(hostname) {
            return MatchResult::Block;
        }

        // 4. Check general blacklist and redirect rules
        for rule in &self.general_rules {
            if self.rule_matches(rule, &ctx) {
                match &rule.action {
                    FilterAction::Block => return MatchResult::Block,
                    FilterAction::Redirect(shim) => return MatchResult::Redirect(shim.clone()),
                    FilterAction::Allow => return MatchResult::Allow,
                    FilterAction::RemoveParam(_) => return MatchResult::Block,
                }
            }
        }

        MatchResult::NoMatch
    }

    fn rule_matches(&self, rule: &NetworkRule, ctx: &RequestContext) -> bool {
        // Party check: 1p vs 3p
        if rule.included_types.contains(RequestTypes::FIRST_PARTY) && ctx.is_third_party {
            return false;
        }
        if rule.included_types.contains(RequestTypes::THIRD_PARTY) && !ctx.is_third_party {
            return false;
        }
        if rule.excluded_types.contains(RequestTypes::FIRST_PARTY) && !ctx.is_third_party {
            return false;
        }
        if rule.excluded_types.contains(RequestTypes::THIRD_PARTY) && ctx.is_third_party {
            return false;
        }

        // Request type check (e.g. script, image)
        let relevant_types = rule.included_types & RequestTypes::ALL_TYPES;
        if !relevant_types.is_empty() && !relevant_types.intersects(ctx.request_type) {
            return false;
        }
        let excluded_relevant = rule.excluded_types & RequestTypes::ALL_TYPES;
        if !excluded_relevant.is_empty() && excluded_relevant.intersects(ctx.request_type) {
            return false;
        }

        // Domain constraints check
        if !rule.included_domains.is_empty() {
            let hit = rule.included_domains.iter().any(|d| ctx.source_domain.ends_with(d));
            if !hit {
                return false;
            }
        }
        if !rule.excluded_domains.is_empty() {
            let hit = rule.excluded_domains.iter().any(|d| ctx.source_domain.ends_with(d));
            if hit {
                return false;
            }
        }

        // Pattern matching
        match &rule.pattern {
            PatternKind::DomainAnchor { domain, path } => {
                let domain_matched = ctx.hostname == domain || ctx.hostname.ends_with(&format!(".{}", domain));
                if !domain_matched {
                    return false;
                }
                if !path.is_empty() {
                    ctx.url.contains(path)
                } else {
                    true
                }
            }
            PatternKind::Substring(needle) => ctx.url.contains(needle),
            PatternKind::LeftAnchor(prefix) => ctx.url.starts_with(prefix),
            PatternKind::RightAnchor(suffix) => ctx.url.ends_with(suffix),
            PatternKind::Exact(exact) => ctx.url == exact,
            PatternKind::Regex(regex) => ctx.url.contains(regex),
        }
    }

    pub fn rule_count(&self) -> usize {
        self.rule_count
    }

    /// Retrieves active cosmetic stylesheet, procedural selectors, and scriptlet injections
    /// for a given domain.
    pub fn retrieve_cosmetics(&self, hostname: &str) -> CosmeticResult {
        self.cosmetic_engine.retrieve(hostname)
    }

    /// Returns the number of cosmetic rules compiled.
    pub fn cosmetic_rule_count(&self) -> usize {
        self.cosmetic_engine.rule_count()
    }

    /// Adds a dynamic matrix firewall rule line (e.g. `* * 3p-script block`).
    pub fn add_dynamic_rule_line(&mut self, line: &str) -> bool {
        self.dynamic_firewall.add_rule_line(line)
    }

    /// Number of dynamic firewall rules currently configured.
    pub fn dynamic_rule_count(&self) -> usize {
        self.dynamic_firewall.rule_count()
    }

    /// Serializes the compiled filtering engine into a binary buffer using bincode.
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserializes a filtering engine from a binary buffer.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    /// Serializes and compresses the compiled engine using LZ4 compression.
    pub fn to_compressed_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        let uncompressed = self.to_bytes()?;
        Ok(lz4_flex::compress_prepend_size(&uncompressed))
    }

    /// Decompresses and deserializes an LZ4-compressed binary buffer.
    pub fn from_compressed_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let decompressed = lz4_flex::decompress_size_prepended(bytes)?;
        let engine = bincode::deserialize(&decompressed)?;
        Ok(engine)
    }

    /// Saves the compiled engine to disk in an LZ4-compressed binary file.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.to_compressed_bytes()?;
        fs::write(path, data)?;
        Ok(())
    }

    /// Loads and decompresses a compiled engine from disk.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read(path)?;
        Self::from_compressed_bytes(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_pure_domain_blocking() {
        let mut engine = StaticNetFilteringEngine::new();
        let list = "
            ! Test List
            ||doubleclick.net^
            ||ads.tracker.org^
            @@||allowed.tracker.org^
        ";
        engine.load_list(list);

        assert_eq!(
            engine.match_request("https://doubleclick.net/ad.js", "example.com", RequestTypes::SCRIPT),
            MatchResult::Block
        );
        assert_eq!(
            engine.match_request("https://sub.doubleclick.net/pixel.gif", "example.com", RequestTypes::IMAGE),
            MatchResult::Block
        );
        assert_eq!(
            engine.match_request("https://allowed.tracker.org/script.js", "example.com", RequestTypes::SCRIPT),
            MatchResult::Allow
        );
        assert_eq!(
            engine.match_request("https://wikipedia.org/wiki/Main_Page", "wikipedia.org", RequestTypes::SUBDOCUMENT),
            MatchResult::NoMatch
        );
    }

    #[test]
    fn test_engine_party_and_type_modifiers() {
        let mut engine = StaticNetFilteringEngine::new();
        let list = "
            ||cdn.adservice.com^$third-party,script
            ||stats.site.com^$1p
        ";
        engine.load_list(list);

        // 3rd-party script -> BLOCK
        assert_eq!(
            engine.match_request("https://cdn.adservice.com/lib.js", "myblog.com", RequestTypes::SCRIPT),
            MatchResult::Block
        );

        // 3rd-party image -> NO MATCH (rule requires script)
        assert_eq!(
            engine.match_request("https://cdn.adservice.com/img.jpg", "myblog.com", RequestTypes::IMAGE),
            MatchResult::NoMatch
        );

        // 1st-party script -> NO MATCH (rule requires 3rd-party)
        assert_eq!(
            engine.match_request("https://cdn.adservice.com/lib.js", "adservice.com", RequestTypes::SCRIPT),
            MatchResult::NoMatch
        );

        // 1st-party stats -> BLOCK
        assert_eq!(
            engine.match_request("https://stats.site.com/track", "site.com", RequestTypes::XMLHTTPREQUEST),
            MatchResult::Block
        );
    }

    #[test]
    fn test_engine_redirect_shim() {
        let mut engine = StaticNetFilteringEngine::new();
        let list = "||google-analytics.com/analytics.js$script,redirect=google-analytics_analytics.js";
        engine.load_list(list);

        assert_eq!(
            engine.match_request("https://google-analytics.com/analytics.js", "example.com", RequestTypes::SCRIPT),
            MatchResult::Redirect("google-analytics_analytics.js".to_string())
        );
    }

    #[test]
    fn test_engine_serialization_roundtrip() {
        let mut engine = StaticNetFilteringEngine::new();
        let list = "
            ||doubleclick.net^
            ||ads.tracker.org^$script
            @@||allowed.tracker.org^
            ||google-analytics.com/analytics.js$script,redirect=google-analytics_analytics.js
        ";
        engine.load_list(list);

        // Serialize and compress
        let compressed = engine.to_compressed_bytes().expect("Failed to compress");
        assert!(!compressed.is_empty());

        // Decompress and deserialize
        let restored = StaticNetFilteringEngine::from_compressed_bytes(&compressed)
            .expect("Failed to restore engine");

        assert_eq!(restored.rule_count(), engine.rule_count());

        // Verify decisions are identical
        assert_eq!(
            restored.match_request("https://doubleclick.net/ad.js", "example.com", RequestTypes::SCRIPT),
            MatchResult::Block
        );
        assert_eq!(
            restored.match_request("https://allowed.tracker.org/script.js", "example.com", RequestTypes::SCRIPT),
            MatchResult::Allow
        );
        assert_eq!(
            restored.match_request("https://google-analytics.com/analytics.js", "example.com", RequestTypes::SCRIPT),
            MatchResult::Redirect("google-analytics_analytics.js".to_string())
        );
        assert_eq!(
            restored.match_request("https://wikipedia.org", "wikipedia.org", RequestTypes::SUBDOCUMENT),
            MatchResult::NoMatch
        );
    }

    #[test]
    fn test_engine_dynamic_matrix_firewall_integration() {
        let mut engine = StaticNetFilteringEngine::new();
        // Whitelist site in static rules
        engine.load_list("@@||safe-site.com^");

        // Globally block all 3p-scripts via dynamic firewall
        engine.add_dynamic_rule_line("* * 3p-script block");
        // Locally allow 3p-script from cdn.safe-site.com on example.com
        engine.add_dynamic_rule_line("example.com cdn.safe-site.com 3p-script allow");

        // 3p script from other host -> blocked by dynamic rule
        assert_eq!(
            engine.match_request("https://cdn.safe-site.com/app.js", "other.com", RequestTypes::SCRIPT),
            MatchResult::Block
        );

        // 3p script from local override -> allowed by local dynamic rule
        assert_eq!(
            engine.match_request("https://cdn.safe-site.com/app.js", "example.com", RequestTypes::SCRIPT),
            MatchResult::Allow
        );
    }
}
