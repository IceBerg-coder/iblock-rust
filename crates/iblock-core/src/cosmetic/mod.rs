//! Cosmetic and Procedural Filtering Engine.
//!
//! Handles CSS element hiding (`##`), cosmetic exceptions (`#@#`), procedural operators
//! (`:has()`, `:has-text()`, `:xpath()`, `:upward()`), and scriptlet injection filters (`##+js(...)`).

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// The category of a cosmetic filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CosmeticKind {
    /// Standard CSS selector (e.g., `.ad-container`, `###banner-ad`).
    PlainCss(String),
    /// Procedural selector containing operators like `:has()`, `:has-text()`, `:matches-path()`.
    Procedural(String),
    /// Injected scriptlet: `##+js(name, args...)`
    Scriptlet {
        name: String,
        args: Vec<String>,
    },
}

/// A parsed cosmetic rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CosmeticRule {
    pub domains: Vec<String>,
    pub excluded_domains: Vec<String>,
    pub is_exception: bool,
    pub kind: CosmeticKind,
}

impl CosmeticRule {
    /// Checks whether this rule applies to a specific hostname.
    pub fn applies_to(&self, hostname: &str) -> bool {
        // Excluded domains override inclusion: e.g. ~example.com
        if self.excluded_domains.iter().any(|d| hostname == d || hostname.ends_with(&format!(".{}", d))) {
            return false;
        }

        // If rule has no domain constraints, it applies generically to all domains
        if self.domains.is_empty() {
            return true;
        }

        // Check if any included domain matches this hostname or a parent domain
        self.domains.iter().any(|d| hostname == d || hostname.ends_with(&format!(".{}", d)))
    }
}

/// Result of evaluating cosmetic filters for a specific web page.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CosmeticResult {
    /// Pre-compiled CSS stylesheet ready for injection via `insertCSS`.
    pub stylesheet: String,
    /// Procedural selectors that require DOM observer evaluation.
    pub procedural_selectors: Vec<String>,
    /// Scriptlets to inject into the page: (name, args)
    pub scriptlets: Vec<(String, Vec<String>)>,
}

/// The Cosmetic & Procedural Filtering Engine.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CosmeticEngine {
    /// Generic plain CSS selectors (no domain constraint): `##.ad`
    generic_css: Vec<String>,
    /// Domain-specific plain CSS selectors: domain -> Vec<selector>
    specific_css: HashMap<String, Vec<String>>,
    /// Unhiding exception selectors: domain -> HashSet<selector>
    exceptions: HashMap<String, HashSet<String>>,
    /// Procedural rules: `:has()`, `:has-text()`, etc.
    procedural_rules: Vec<CosmeticRule>,
    /// Scriptlet injection rules: `##+js(...)`
    scriptlet_rules: Vec<CosmeticRule>,
    /// Total count of loaded cosmetic rules
    rule_count: usize,
}

impl CosmeticEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a raw cosmetic line (e.g. `example.com##.ad`, `###banner`, `example.com#@#.ad`)
    /// and adds it to the engine.
    pub fn add_line(&mut self, line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('[') {
            return false;
        }

        // Determine delimiter: ## (hiding), #@# (exception), #?# (procedural marker)
        let (domains_str, is_exception, body) = if let Some(idx) = trimmed.find("#@#") {
            (&trimmed[..idx], true, &trimmed[idx + 3..])
        } else if let Some(idx) = trimmed.find("##") {
            (&trimmed[..idx], false, &trimmed[idx + 2..])
        } else if let Some(idx) = trimmed.find("#?#") {
            (&trimmed[..idx], false, &trimmed[idx + 3..])
        } else {
            return false;
        };

        if body.is_empty() {
            return false;
        }

        // Parse domains
        let mut domains = Vec::new();
        let mut excluded_domains = Vec::new();
        if !domains_str.is_empty() {
            for d in domains_str.split(',') {
                let d = d.trim();
                if let Some(excluded) = d.strip_prefix('~') {
                    if !excluded.is_empty() {
                        excluded_domains.push(excluded.to_lowercase());
                    }
                } else if !d.is_empty() {
                    domains.push(d.to_lowercase());
                }
            }
        }

        // Determine rule kind
        let kind = if let Some(scriptlet_body) = body.strip_prefix("+js(") {
            let inner = scriptlet_body.trim_end_matches(')');
            let mut parts = inner.split(',').map(|s| s.trim().to_string());
            let name = parts.next().unwrap_or_default();
            let args = parts.collect();
            CosmeticKind::Scriptlet { name, args }
        } else if is_procedural_selector(body) {
            CosmeticKind::Procedural(body.to_string())
        } else {
            CosmeticKind::PlainCss(body.to_string())
        };

        self.rule_count += 1;

        if is_exception {
            if let CosmeticKind::PlainCss(ref sel) = kind {
                for d in &domains {
                    self.exceptions.entry(d.clone()).or_default().insert(sel.clone());
                }
                return true;
            }
        }

        match kind {
            CosmeticKind::PlainCss(ref sel) => {
                if domains.is_empty() && excluded_domains.is_empty() {
                    self.generic_css.push(sel.clone());
                } else {
                    for d in &domains {
                        self.specific_css.entry(d.clone()).or_default().push(sel.clone());
                    }
                }
            }
            CosmeticKind::Procedural(_) => {
                self.procedural_rules.push(CosmeticRule {
                    domains,
                    excluded_domains,
                    is_exception,
                    kind,
                });
            }
            CosmeticKind::Scriptlet { .. } => {
                self.scriptlet_rules.push(CosmeticRule {
                    domains,
                    excluded_domains,
                    is_exception,
                    kind,
                });
            }
        }

        true
    }

    /// Evaluates all cosmetic, procedural, and scriptlet rules for the target hostname.
    ///
    /// Generates the consolidated CSS string with `{ display: none !important; }`
    /// minus any active exceptions for that domain.
    pub fn retrieve(&self, hostname: &str) -> CosmeticResult {
        let mut active_selectors: HashSet<&str> = HashSet::new();

        // 1. Collect generic CSS selectors
        for sel in &self.generic_css {
            active_selectors.insert(sel.as_str());
        }

        // 2. Collect domain-specific CSS selectors (domain + parent domains)
        let domain_parts = domain_hierarchy(hostname);
        for d in &domain_parts {
            if let Some(list) = self.specific_css.get(*d) {
                for sel in list {
                    active_selectors.insert(sel.as_str());
                }
            }
        }

        // 3. Remove exceptions (#@#)
        for d in &domain_parts {
            if let Some(exc_set) = self.exceptions.get(*d) {
                for exc in exc_set {
                    active_selectors.remove(exc.as_str());
                }
            }
        }

        // 4. Build consolidated CSS stylesheet
        let stylesheet = if active_selectors.is_empty() {
            String::new()
        } else {
            let mut selectors: Vec<&str> = active_selectors.into_iter().collect();
            selectors.sort_unstable(); // Deterministic output
            format!("{} {{ display: none !important; }}", selectors.join(",\n"))
        };

        // 5. Collect procedural selectors
        let mut procedural_selectors = Vec::new();
        for rule in &self.procedural_rules {
            if rule.applies_to(hostname) {
                if let CosmeticKind::Procedural(ref sel) = rule.kind {
                    procedural_selectors.push(sel.clone());
                }
            }
        }

        // 6. Collect matching scriptlet injections
        let mut scriptlets = Vec::new();
        for rule in &self.scriptlet_rules {
            if rule.applies_to(hostname) {
                if let CosmeticKind::Scriptlet { ref name, ref args } = rule.kind {
                    scriptlets.push((name.clone(), args.clone()));
                }
            }
        }

        CosmeticResult {
            stylesheet,
            procedural_selectors,
            scriptlets,
        }
    }

    pub fn rule_count(&self) -> usize {
        self.rule_count
    }
}

/// Checks if a selector contains procedural operators.
fn is_procedural_selector(selector: &str) -> bool {
    selector.contains(":has(")
        || selector.contains(":has-text(")
        || selector.contains(":matches-path(")
        || selector.contains(":matches-css(")
        || selector.contains(":xpath(")
        || selector.contains(":upward(")
        || selector.contains(":remove()")
}

/// Generates all domain components in hierarchy:
/// `sub.example.com` -> `["sub.example.com", "example.com"]`
fn domain_hierarchy(hostname: &str) -> Vec<&str> {
    let mut domains = Vec::new();
    let mut current = hostname;
    domains.push(current);

    while let Some(dot_idx) = current.find('.') {
        current = &current[dot_idx + 1..];
        if !current.contains('.') {
            break;
        }
        domains.push(current);
    }
    domains
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_css_generic_and_specific() {
        let mut engine = CosmeticEngine::new();
        engine.add_line("##.ad-banner");
        engine.add_line("###sponsor-box");
        engine.add_line("example.com##.sidebar-promo");

        let res_example = engine.retrieve("example.com");
        assert!(res_example.stylesheet.contains(".ad-banner"));
        assert!(res_example.stylesheet.contains("#sponsor-box"));
        assert!(res_example.stylesheet.contains(".sidebar-promo"));

        let res_other = engine.retrieve("wikipedia.org");
        assert!(res_other.stylesheet.contains(".ad-banner"));
        assert!(!res_other.stylesheet.contains(".sidebar-promo"));
    }

    #[test]
    fn test_cosmetic_unhiding_exception() {
        let mut engine = CosmeticEngine::new();
        engine.add_line("##.promo");
        engine.add_line("example.com#@#.promo");

        let res_other = engine.retrieve("other.com");
        assert!(res_other.stylesheet.contains(".promo"));

        let res_example = engine.retrieve("example.com");
        assert!(!res_example.stylesheet.contains(".promo"));
    }

    #[test]
    fn test_procedural_selector_dispatch() {
        let mut engine = CosmeticEngine::new();
        engine.add_line("example.com##div:has(> .sponsored)");
        engine.add_line("example.com##article:has-text(Promoted)");

        let res = engine.retrieve("example.com");
        assert_eq!(res.procedural_selectors.len(), 2);
        assert!(res.procedural_selectors.contains(&"div:has(> .sponsored)".to_string()));
        assert!(res.procedural_selectors.contains(&"article:has-text(Promoted)".to_string()));
    }

    #[test]
    fn test_scriptlet_injection_rule() {
        let mut engine = CosmeticEngine::new();
        engine.add_line("example.com##+js(set, ads_active, false)");

        let res = engine.retrieve("example.com");
        assert_eq!(res.scriptlets.len(), 1);
        let (ref name, ref args) = res.scriptlets[0];
        assert_eq!(name, "set");
        assert_eq!(args, &vec!["ads_active".to_string(), "false".to_string()]);
    }
}
