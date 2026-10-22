//! # iBlock Core
//!
//! High-performance, memory-efficient content blocking and static network filtering
//! engine ported from uBlock Origin (uBO).
//!
//! Includes:
//! - Contiguous Radix Host Name Trie ([`HNTrie`](crate::trie::HNTrie))
//! - Bidirectional Pattern Trie ([`BidiTrie`](crate::trie::BidiTrie))
//! - Adblock Plus & uBlock Origin zero-copy filter parser
//! - Static Network Filtering Engine ([`StaticNetFilteringEngine`](crate::engine::StaticNetFilteringEngine))

pub mod cosmetic;
pub mod dynamic;
pub mod engine;
pub mod filter;
pub mod parser;
pub mod surrogate;
pub mod trie;

pub use cosmetic::{CosmeticEngine, CosmeticKind, CosmeticResult, CosmeticRule};
pub use dynamic::{DynamicAction, DynamicFirewall, DynamicRule, DynamicType};
pub use engine::{MatchResult, RequestContext, StaticNetFilteringEngine};
pub use filter::{FilterAction, NetworkRule, PatternKind, RequestTypes};
pub use parser::{parse_line, ParsedLine};
pub use surrogate::{lookup as lookup_surrogate, resolve_data_uri as resolve_surrogate_data_uri, Surrogate};
pub use trie::{BidiTrie, BidiTrieContainer, HNTrie, HNTrieContainer};
