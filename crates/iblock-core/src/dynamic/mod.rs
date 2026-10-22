//! Dynamic Matrix Firewall.
//!
//! Implements uBlock Origin's two-dimensional point-and-click firewall.
//! Supports global (`*`) and per-site rules across resource types (`3p-script`, `3p-frame`,
//! `image`, `inline-script`), with `block`, `allow`, and `noop` actions.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Dynamic filtering action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicAction {
    Block = 1,
    Allow = 2,
    Noop = 3,
}

/// Dynamic filtering resource type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DynamicType {
    Any,
    ThirdParty,
    Image,
    InlineScript,
    FirstPartyScript,
    ThirdPartyScript,
    ThirdPartyFrame,
}

impl DynamicType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "*" | "any" => Some(DynamicType::Any),
            "3p" => Some(DynamicType::ThirdParty),
            "image" => Some(DynamicType::Image),
            "inline-script" => Some(DynamicType::InlineScript),
            "1p-script" => Some(DynamicType::FirstPartyScript),
            "3p-script" => Some(DynamicType::ThirdPartyScript),
            "3p-frame" | "frame" => Some(DynamicType::ThirdPartyFrame),
            _ => None,
        }
    }
}

/// A parsed dynamic matrix firewall rule: `<source> <target> <type> <action>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicRule {
    pub source: String,
    pub target: String,
    pub dynamic_type: DynamicType,
    pub action: DynamicAction,
}

/// Two-dimensional matrix firewall holding source x target rules.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DynamicFirewall {
    /// Rules indexed by `(source, target, dynamic_type)`
    rules: HashMap<(String, String, DynamicType), DynamicAction>,
    rule_count: usize,
}

impl DynamicFirewall {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a rule: `add_rule("example.com", "google.com", DynamicType::ThirdPartyScript, DynamicAction::Block)`.
    pub fn add_rule(&mut self, source: &str, target: &str, dtype: DynamicType, action: DynamicAction) {
        let key = (source.to_lowercase(), target.to_lowercase(), dtype);
        if self.rules.insert(key, action).is_none() {
            self.rule_count += 1;
        }
    }

    /// Parses and adds a rule line: e.g. `* * 3p-script block` or `example.com cdn.com * allow`.
    pub fn add_rule_line(&mut self, line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return false;
        }

        let source = parts[0];
        let target = parts[1];
        let dtype = match DynamicType::parse(parts[2]) {
            Some(t) => t,
            None => return false,
        };

        let action = match parts[3] {
            "block" => DynamicAction::Block,
            "allow" => DynamicAction::Allow,
            "noop" => DynamicAction::Noop,
            _ => return false,
        };

        self.add_rule(source, target, dtype, action);
        true
    }

    /// Evaluates dynamic rules for a given request context.
    ///
    /// Follows uBO's specificity hierarchy:
    /// 1. `(source, target)`
    /// 2. `(source, *)`
    /// 3. `(*, target)`
    /// 4. `(*, *)`
    /// At each scope level, specific type precedes `DynamicType::Any`.
    pub fn evaluate(&self, source_domain: &str, target_domain: &str, dtype: DynamicType) -> Option<DynamicAction> {
        let scopes = [
            (source_domain, target_domain),
            (source_domain, "*"),
            ("*", target_domain),
            ("*", "*"),
        ];

        for (src, tgt) in scopes {
            // First check specific type
            if dtype != DynamicType::Any {
                let key = (src.to_string(), tgt.to_string(), dtype);
                if let Some(&action) = self.rules.get(&key) {
                    return Some(action);
                }
            }

            // Fallback to generic `Any` type at this scope level
            let any_key = (src.to_string(), tgt.to_string(), DynamicType::Any);
            if let Some(&action) = self.rules.get(&any_key) {
                return Some(action);
            }
        }

        None
    }

    pub fn rule_count(&self) -> usize {
        self.rule_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_3p_script_block() {
        let mut firewall = DynamicFirewall::new();
        firewall.add_rule_line("* * 3p-script block");

        // Global 3p-script blocked
        assert_eq!(
            firewall.evaluate("nytimes.com", "tracker.com", DynamicType::ThirdPartyScript),
            Some(DynamicAction::Block)
        );

        // Images still allowed (no rule)
        assert_eq!(
            firewall.evaluate("nytimes.com", "tracker.com", DynamicType::Image),
            None
        );
    }

    #[test]
    fn test_local_override_global() {
        let mut firewall = DynamicFirewall::new();
        // Globally block all 3p frames
        firewall.add_rule_line("* * 3p-frame block");
        // Locally allow frames from youtube on specific site
        firewall.add_rule_line("example.com youtube.com 3p-frame allow");

        // Other site -> blocked
        assert_eq!(
            firewall.evaluate("other.com", "youtube.com", DynamicType::ThirdPartyFrame),
            Some(DynamicAction::Block)
        );

        // Local override -> allowed
        assert_eq!(
            firewall.evaluate("example.com", "youtube.com", DynamicType::ThirdPartyFrame),
            Some(DynamicAction::Allow)
        );
    }

    #[test]
    fn test_noop_override() {
        let mut firewall = DynamicFirewall::new();
        firewall.add_rule_line("* * 3p block");
        // Set noop for trusted CDN
        firewall.add_rule_line("example.com cdnjs.cloudflare.com * noop");

        assert_eq!(
            firewall.evaluate("example.com", "cdnjs.cloudflare.com", DynamicType::ThirdPartyScript),
            Some(DynamicAction::Noop)
        );
    }
}
