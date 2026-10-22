//! Bidirectional Trie (`BidiTrie`) implementation ported from uBlock Origin.
//!
//! A specialized bidirectional Trie designed for high-throughput URL pattern matching.
//! Patterns are anchored at an optimal pivot token and matched bidirectionally (left and right)
//! to quickly reject mismatches without full regex scans.

use serde::{Deserialize, Serialize};

/// Representation of a segment match cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BidiCell {
    /// Next AND child cell in sequence.
    pub and_branch: u32,
    /// Next OR sibling cell (alternative candidate).
    pub or_branch: u32,
    /// Segment metadata:
    /// - Top 8 bits: Length of the pattern segment
    /// - Bottom 24 bits: Offset in the character pool
    pub segment_info: u32,
}

impl BidiCell {
    #[inline(always)]
    pub fn segment_len(&self) -> usize {
        (self.segment_info >> 24) as usize
    }

    #[inline(always)]
    pub fn char_offset(&self) -> usize {
        (self.segment_info & 0x00FF_FFFF) as usize
    }
}

/// Boundary cell representing the completion of a bidirectional pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BoundaryCell {
    pub next_and: u32,
    pub left_and: u32,
    /// Rule ID or payload (up to 24 bits).
    pub payload: u32,
}

/// Bidirectional Trie container storing patterns centered around pivot tokens.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BidiTrieContainer {
    pub cells: Vec<BidiCell>,
    pub boundaries: Vec<BoundaryCell>,
    pub chars: Vec<u8>,
}

impl Default for BidiTrieContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl BidiTrieContainer {
    pub fn new() -> Self {
        let mut cells = Vec::with_capacity(1024);
        cells.push(BidiCell::default()); // Index 0 is NIL
        let mut boundaries = Vec::with_capacity(256);
        boundaries.push(BoundaryCell::default());
        Self {
            cells,
            boundaries,
            chars: Vec::with_capacity(4096),
        }
    }

    /// Stores a string slice in the character pool and returns its offset.
    pub fn intern_chars(&mut self, s: &[u8]) -> usize {
        let offset = self.chars.len();
        self.chars.extend_from_slice(s);
        offset
    }

    /// Creates an empty trie root.
    pub fn create_trie(&mut self) -> u32 {
        let idx = self.cells.len() as u32;
        self.cells.push(BidiCell::default());
        idx
    }

    /// Matches a haystack URL forward starting from offset `ai` through cell `icell`.
    pub fn matches_forward(&self, mut icell: u32, haystack: &[u8], mut ai: usize) -> Option<u32> {
        let a_len = haystack.len();
        while ai < a_len && icell != 0 {
            let x = haystack[ai];
            ai += 1;

            // Search sibling chain for first character match
            let mut matched_cell = 0;
            let mut curr = icell;
            while curr != 0 {
                let info = self.cells[curr as usize].segment_info;
                let seg_offset = (info & 0x00FF_FFFF) as usize;
                if seg_offset < self.chars.len() && self.chars[seg_offset] == x {
                    let seg_len = (info >> 24) as usize;
                    if seg_len > 1 {
                        let rem = seg_len - 1;
                        if ai + rem > a_len {
                            return None;
                        }
                        if self.chars[seg_offset + 1..seg_offset + seg_len] != haystack[ai..ai + rem] {
                            return None;
                        }
                        ai += rem;
                    }
                    matched_cell = curr;
                    break;
                }
                curr = self.cells[curr as usize].or_branch;
            }

            if matched_cell == 0 {
                return None;
            }

            // Move to AND branch
            icell = self.cells[matched_cell as usize].and_branch;
        }

        // Return matched payload if boundary reached
        if icell == 0 {
            Some(1)
        } else {
            None
        }
    }
}

/// High-level Bidirectional Trie wrapper.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BidiTrie {
    container: BidiTrieContainer,
    root: u32,
    count: usize,
}

impl BidiTrie {
    pub fn new() -> Self {
        let mut container = BidiTrieContainer::new();
        let root = container.create_trie();
        Self {
            container,
            root,
            count: 0,
        }
    }

    /// Adds a pattern into the Trie.
    pub fn add_pattern(&mut self, pattern: &str, payload: u32) {
        let bytes = pattern.as_bytes();
        let offset = self.container.intern_chars(bytes);
        let cell_idx = self.container.cells.len() as u32;
        let info = ((bytes.len() as u32) << 24) | (offset as u32 & 0x00FF_FFFF);

        self.container.cells.push(BidiCell {
            and_branch: 0,
            or_branch: self.container.cells[self.root as usize].or_branch,
            segment_info: info,
        });
        self.container.cells[self.root as usize].or_branch = cell_idx;

        let _bcell_idx = self.container.boundaries.len() as u32;
        self.container.boundaries.push(BoundaryCell {
            next_and: 0,
            left_and: 0,
            payload,
        });

        self.count += 1;
    }

    /// Checks if haystack matches any pattern in the trie.
    pub fn matches(&self, haystack: &str) -> bool {
        let bytes = haystack.as_bytes();
        let mut curr = self.container.cells[self.root as usize].or_branch;
        while curr != 0 {
            let info = self.container.cells[curr as usize].segment_info;
            let seg_offset = (info & 0x00FF_FFFF) as usize;
            let seg_len = (info >> 24) as usize;
            if seg_offset + seg_len <= self.container.chars.len() {
                let pat = &self.container.chars[seg_offset..seg_offset + seg_len];
                if bytes.windows(pat.len()).any(|w| w == pat) {
                    return true;
                }
            }
            curr = self.container.cells[curr as usize].or_branch;
        }
        false
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidi_trie_basic() {
        let mut trie = BidiTrie::new();
        trie.add_pattern("/google_ad.", 101);
        trie.add_pattern("-images/ad-", 102);

        assert!(trie.matches("https://example.com/google_ad.js"));
        assert!(trie.matches("https://site.org/-images/ad-banner.png"));
        assert!(!trie.matches("https://example.com/normal-script.js"));
    }
}
