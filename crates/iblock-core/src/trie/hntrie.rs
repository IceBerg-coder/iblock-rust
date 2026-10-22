//! Hostname Trie (`HNTrie`) implementation ported from uBlock Origin.
//!
//! A high-performance Radix (Patricia) Trie specialized for domain and hostname matching.
//! Hostnames are stored and traversed in reverse order (e.g., `com.example.ads`), allowing
//! instantaneous subdomain matching (e.g., `sub.ads.example.com` matches `ads.example.com`).

use serde::{Deserialize, Serialize};

const INITIAL_CAPACITY_CELLS: usize = 1024;
const INITIAL_CHAR_CAPACITY: usize = 4096;

/// A single cell in the flat HNTrie array.
/// Stored in 12 bytes (3 x u32) to match uBO's cache-friendly layout:
/// - `sibling`: Index of the next sibling cell (alternative branch).
/// - `child`: Index of the first child cell (subdomain continuation).
/// - `info`: Segment info:
///     - bits [0..7]: Segment length (1 to 127) and boundary bit (0x80)
///     - bits [8..31]: Char buffer byte offset for the segment
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Cell {
    pub sibling: u32,
    pub child: u32,
    pub info: u32,
}

impl Cell {
    #[inline(always)]
    pub fn is_boundary(&self) -> bool {
        (self.info & 0x80) != 0
    }

    #[inline(always)]
    pub fn segment_len(&self) -> usize {
        (self.info & 0x7F) as usize
    }

    #[inline(always)]
    pub fn char_offset(&self) -> usize {
        (self.info >> 8) as usize
    }

    #[inline(always)]
    pub fn set_boundary(&mut self) {
        self.info |= 0x80;
    }
}

/// A compact Hostname Trie container.
/// Multiple distinct tries can share the same underlying container buffer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HNTrieContainer {
    pub cells: Vec<Cell>,
    pub chars: Vec<u8>,
}

impl Default for HNTrieContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl HNTrieContainer {
    /// Creates a new HNTrie container with pre-allocated buffers.
    pub fn new() -> Self {
        let mut cells = Vec::with_capacity(INITIAL_CAPACITY_CELLS);
        // Cell 0 is reserved as the null pointer (NIL)
        cells.push(Cell::default());
        Self {
            cells,
            chars: Vec::with_capacity(INITIAL_CHAR_CAPACITY),
        }
    }

    /// Allocates an empty trie root and returns its handle (cell index).
    pub fn create_trie(&mut self) -> u32 {
        let idx = self.cells.len() as u32;
        self.cells.push(Cell::default());
        idx
    }

    /// Adds a reversed character slice into the character pool.
    fn append_reversed_chars(&mut self, bytes: &[u8]) -> usize {
        let offset = self.chars.len();
        self.chars.reserve(bytes.len());
        for &b in bytes.iter().rev() {
            self.chars.push(b);
        }
        offset
    }

    /// Allocates a new leaf cell representing the remaining reversed hostname bytes.
    fn alloc_leaf_cell(&mut self, bytes: &[u8]) -> u32 {
        let len = bytes.len();
        let offset = self.append_reversed_chars(bytes);
        let idx = self.cells.len() as u32;
        let info = ((offset as u32) << 8) | 0x80 | (len as u32 & 0x7F);
        self.cells.push(Cell {
            sibling: 0,
            child: 0,
            info,
        });
        idx
    }

    /// Allocates a new intermediate cell.
    fn alloc_cell(&mut self, sibling: u32, child: u32, char_offset: usize, len: usize, boundary: bool) -> u32 {
        let idx = self.cells.len() as u32;
        let boundary_bit = if boundary { 0x80 } else { 0x00 };
        let info = ((char_offset as u32) << 8) | boundary_bit | (len as u32 & 0x7F);
        self.cells.push(Cell {
            sibling,
            child,
            info,
        });
        idx
    }

    /// Adds a hostname to the specified trie root.
    ///
    /// Returns:
    /// - `1` if a new hostname was added
    /// - `0` if the hostname was already present
    /// - `-1` if a shorter domain is already present that subsumes this hostname
    pub fn add(&mut self, root: u32, hostname: &str) -> i32 {
        let bytes = hostname.as_bytes();
        if bytes.is_empty() {
            return 0;
        }

        let mut rem_len = bytes.len();
        let mut icell = self.cells[root as usize].child;

        // If trie root is empty, insert first leaf cell
        if icell == 0 {
            let leaf = self.alloc_leaf_cell(&bytes[..rem_len]);
            self.cells[root as usize].child = leaf;
            return 1;
        }

        loop {
            let v = self.cells[icell as usize].info;
            let seg_offset = (v >> 8) as usize;
            let first_char = self.chars[seg_offset];

            // Check if first character matches (traversing reversed bytes)
            if first_char != bytes[rem_len - 1] {
                let next_sibling = self.cells[icell as usize].sibling;
                if next_sibling == 0 {
                    let leaf = self.alloc_leaf_cell(&bytes[..rem_len]);
                    self.cells[icell as usize].sibling = leaf;
                    return 1;
                }
                icell = next_sibling;
                continue;
            }

            // 1st character matched
            let mut seg_match = 1;
            rem_len -= 1;
            let seg_len = (v & 0x7F) as usize;

            if seg_len > 1 {
                while seg_match < seg_len && rem_len > 0 {
                    if self.chars[seg_offset + seg_match] != bytes[rem_len - 1] {
                        break;
                    }
                    seg_match += 1;
                    rem_len -= 1;
                }
            }

            let boundary_bit = (v & 0x80) != 0;

            // Entire segment matched
            if seg_match == seg_len {
                if rem_len == 0 {
                    // Needle exhausted
                    if boundary_bit {
                        return 0; // Already present
                    }
                    self.cells[icell as usize].set_boundary();
                    return 1;
                } else {
                    // Needle remainder exists
                    if boundary_bit && bytes[rem_len - 1] == b'.' {
                        // Subdomain already subsumed by an existing broader rule
                        return -1;
                    }
                    let child = self.cells[icell as usize].child;
                    if child != 0 {
                        icell = child;
                        continue;
                    }
                    let leaf = self.alloc_leaf_cell(&bytes[..rem_len]);
                    self.cells[icell as usize].child = leaf;
                    return 1;
                }
            } else {
                // Partial segment match: need to split the current cell
                let old_child = self.cells[icell as usize].child;

                // Update current cell to hold the matched prefix
                self.cells[icell as usize].info = ((seg_offset as u32) << 8) | (seg_match as u32 & 0x7F);

                // Allocate intermediate cell for the remainder of the original segment
                let split_cell = self.alloc_cell(
                    0,
                    old_child,
                    seg_offset + seg_match,
                    seg_len - seg_match,
                    boundary_bit,
                );
                self.cells[icell as usize].child = split_cell;

                if rem_len != 0 {
                    // Needle has remainder: add new leaf as sibling of the split cell
                    let leaf = self.alloc_leaf_cell(&bytes[..rem_len]);
                    self.cells[split_cell as usize].sibling = leaf;
                } else {
                    // Needle exhausted at the split point
                    self.cells[icell as usize].set_boundary();
                }
                return 1;
            }
        }
    }

    /// Tests whether the given hostname matches any rule in the trie.
    ///
    /// If a match is found, returns `Some(index)` where `index` is the start position
    /// in `hostname` of the matching domain/subdomain rule.
    ///
    /// Examples:
    /// - Querying `abc.com` against rule `abc.com` returns `Some(0)` (exact match).
    /// - Querying `www.abc.com` against rule `abc.com` returns `Some(4)` (subdomain match).
    /// - Querying `notabc.com` against rule `abc.com` returns `None`.
    pub fn matches(&self, root: u32, hostname: &str) -> Option<usize> {
        let bytes = hostname.as_bytes();
        let mut ineedle = bytes.len();
        let mut icell = self.cells[root as usize].child;

        if icell == 0 || ineedle == 0 {
            return None;
        }

        loop {
            if ineedle == 0 {
                return None;
            }
            ineedle -= 1;
            let c = bytes[ineedle];

            // Find first matching sibling
            loop {
                let v = self.cells[icell as usize].info;
                let seg_offset = (v >> 8) as usize;
                if self.chars[seg_offset] == c {
                    break;
                }
                icell = self.cells[icell as usize].sibling;
                if icell == 0 {
                    return None;
                }
            }

            // Verify the rest of the segment
            let v = self.cells[icell as usize].info;
            let mut seg_len = (v & 0x7F) as usize;
            let mut seg_offset = (v >> 8) as usize;

            if seg_len > 1 {
                seg_len -= 1;
                if seg_len > ineedle {
                    return None;
                }
                seg_offset += 1;
                for _ in 0..seg_len {
                    ineedle -= 1;
                    if self.chars[seg_offset] != bytes[ineedle] {
                        return None;
                    }
                    seg_offset += 1;
                }
            }

            // Boundary check: is this a valid domain termination?
            if (v & 0x80) != 0 {
                if ineedle == 0 || bytes[ineedle - 1] == b'.' {
                    return Some(ineedle);
                }
            }

            // Advance to next segment (subdomain continuation)
            icell = self.cells[icell as usize].child;
            if icell == 0 {
                break;
            }
        }

        None
    }

    /// Checks if the query hostname matches any rule in the trie.
    #[inline(always)]
    pub fn contains(&self, root: u32, hostname: &str) -> bool {
        self.matches(root, hostname).is_some()
    }
}

/// A standalone Host Name Trie owning its own container.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HNTrie {
    container: HNTrieContainer,
    root: u32,
    count: usize,
}

impl HNTrie {
    pub fn new() -> Self {
        let mut container = HNTrieContainer::new();
        let root = container.create_trie();
        Self {
            container,
            root,
            count: 0,
        }
    }

    pub fn from_iter<'a, I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut trie = Self::new();
        for hn in iter {
            trie.add(hn);
        }
        trie
    }

    pub fn add(&mut self, hostname: &str) -> bool {
        let res = self.container.add(self.root, hostname);
        if res == 1 {
            self.count += 1;
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn matches<'a>(&self, hostname: &'a str) -> Option<&'a str> {
        self.container.matches(self.root, hostname).map(|idx| &hostname[idx..])
    }

    #[inline(always)]
    pub fn contains(&self, hostname: &str) -> bool {
        self.container.contains(self.root, hostname)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_trie() {
        let trie = HNTrie::new();
        assert!(!trie.contains("example.com"));
        assert_eq!(trie.matches("example.com"), None);
    }

    #[test]
    fn test_exact_and_subdomain_matches() {
        let mut trie = HNTrie::new();
        assert!(trie.add("example.com"));
        assert!(trie.add("doubleclick.net"));

        // Exact match
        assert!(trie.contains("example.com"));
        assert_eq!(trie.matches("example.com"), Some("example.com"));

        // Subdomain match
        assert!(trie.contains("ads.example.com"));
        assert_eq!(trie.matches("ads.example.com"), Some("example.com"));

        assert!(trie.contains("a.b.c.doubleclick.net"));
        assert_eq!(trie.matches("a.b.c.doubleclick.net"), Some("doubleclick.net"));

        // Partial match that is NOT a subdomain should NOT match
        assert!(!trie.contains("notexample.com"));
        assert_eq!(trie.matches("notexample.com"), None);

        assert!(!trie.contains("example.org"));
        assert_eq!(trie.matches("example.org"), None);

        assert!(!trie.contains("com"));
    }

    #[test]
    fn test_subdomain_subsumption() {
        let mut trie = HNTrie::new();
        assert!(trie.add("example.com"));
        // Adding a subdomain when base domain is already in the trie returns -1 (redundant)
        assert!(!trie.add("ads.example.com"));

        // Duplicate
        assert!(!trie.add("example.com"));
    }

    #[test]
    fn test_multiple_tlds_and_prefixes() {
        let mut trie = HNTrie::new();
        trie.add("google.com");
        trie.add("google.co.uk");
        trie.add("google.de");
        trie.add("analytics.google.com");

        assert!(trie.contains("mail.google.com"));
        assert!(trie.contains("maps.google.co.uk"));
        assert!(trie.contains("google.de"));
        assert!(!trie.contains("google.fr"));
        assert!(!trie.contains("evilgoogle.com"));
    }
}
