//! Data types representing parsed static content-blocking rules and modifiers.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Contextual resource types and party flags for matching HTTP requests.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct RequestTypes: u32 {
        const SCRIPT           = 1 << 0;
        const IMAGE            = 1 << 1;
        const STYLESHEET       = 1 << 2;
        const XMLHTTPREQUEST   = 1 << 3;
        const SUBDOCUMENT      = 1 << 4;
        const WEBSOCKET        = 1 << 5;
        const MEDIA            = 1 << 6;
        const FONT             = 1 << 7;
        const OTHER            = 1 << 8;
        const FIRST_PARTY      = 1 << 9;
        const THIRD_PARTY      = 1 << 10;
        const IMPORTANT        = 1 << 11;
        const MATCH_CASE       = 1 << 12;

        const ALL_TYPES = Self::SCRIPT.bits()
            | Self::IMAGE.bits()
            | Self::STYLESHEET.bits()
            | Self::XMLHTTPREQUEST.bits()
            | Self::SUBDOCUMENT.bits()
            | Self::WEBSOCKET.bits()
            | Self::MEDIA.bits()
            | Self::FONT.bits()
            | Self::OTHER.bits();
    }
}

/// Action to apply when a rule matches.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterAction {
    /// Disallow the request
    Block,
    /// Allow the request (whitelist / exception)
    Allow,
    /// Redirect request to a neutered local surrogate shim
    Redirect(String),
    /// Strip specific query parameters
    RemoveParam(String),
}

/// The pattern matching strategy for the rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternKind {
    /// Domain anchor with optional path: `||domain.com^` or `||domain.com/path`
    DomainAnchor { domain: String, path: String },
    /// Substring plain match
    Substring(String),
    /// Left anchored: `|http://...`
    LeftAnchor(String),
    /// Right anchored: `.../banner.png|`
    RightAnchor(String),
    /// Exact complete URL anchor: `|http://...|`
    Exact(String),
    /// Regular expression
    Regex(String),
}

/// A parsed static network filter rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRule {
    pub pattern: PatternKind,
    pub action: FilterAction,
    pub included_types: RequestTypes,
    pub excluded_types: RequestTypes,
    pub included_domains: Vec<String>,
    pub excluded_domains: Vec<String>,
}

impl NetworkRule {
    /// Returns true if this rule is an exception / allow rule.
    #[inline(always)]
    pub fn is_exception(&self) -> bool {
        matches!(self.action, FilterAction::Allow)
    }

    /// Returns true if this rule has the `$important` modifier.
    #[inline(always)]
    pub fn is_important(&self) -> bool {
        self.included_types.contains(RequestTypes::IMPORTANT)
    }
}
