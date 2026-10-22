//! Surrogate scriptlets and neutral shims registry.
//!
//! Provides embedded, zero-allocation neutral shims (such as Google Tag Manager,
//! Google Analytics, empty images, no-op audio) to safely defuse tracking scripts
//! without breaking host page execution.

/// A registered neutral surrogate resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Surrogate {
    pub name: &'static str,
    pub mime_type: &'static str,
    pub content: &'static str,
}

impl Surrogate {
    /// Encodes this surrogate as a ready-to-use Data URI.
    pub fn to_data_uri(&self) -> String {
        format!("data:{};utf8,{}", self.mime_type, self.content)
    }
}

pub const EMPTY_GIF_DATA_URI: &str = "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";
pub const NOOP_JS: &str = "(() => {})();";

/// Static catalog of common surrogates.
static SURROGATES: &[Surrogate] = &[
    Surrogate {
        name: "noopjs",
        mime_type: "application/javascript",
        content: NOOP_JS,
    },
    Surrogate {
        name: "noop.js",
        mime_type: "application/javascript",
        content: NOOP_JS,
    },
    Surrogate {
        name: "nooptext",
        mime_type: "text/plain",
        content: "",
    },
    Surrogate {
        name: "noop.txt",
        mime_type: "text/plain",
        content: "",
    },
    Surrogate {
        name: "1x1.gif",
        mime_type: "image/gif",
        content: EMPTY_GIF_DATA_URI,
    },
    Surrogate {
        name: "google-analytics_analytics.js",
        mime_type: "application/javascript",
        content: "(function() { window.ga = window.ga || function() {}; window.ga.loaded = true; })();",
    },
    Surrogate {
        name: "google-analytics_ga.js",
        mime_type: "application/javascript",
        content: "(function() { window._gaq = window._gaq || []; window._gat = { _getTrackerByName: function() { return { _trackPageview: function() {} }; } }; })();",
    },
    Surrogate {
        name: "googletagmanager_gtm.js",
        mime_type: "application/javascript",
        content: "(function() { window.dataLayer = window.dataLayer || []; window.dataLayer.push = function(o) { if (o && typeof o.eventCallback === 'function') setTimeout(o.eventCallback, 1); }; })();",
    },
    Surrogate {
        name: "googlesyndication_adsbygoogle.js",
        mime_type: "application/javascript",
        content: "(function() { window.adsbygoogle = window.adsbygoogle || []; window.adsbygoogle.loaded = true; window.adsbygoogle.push = function() {}; })();",
    },
    Surrogate {
        name: "amazon_ads.js",
        mime_type: "application/javascript",
        content: "(function() { window.amznads = window.amznads || {}; })();",
    },
];

/// Looks up a surrogate by resource name or alias.
pub fn lookup(name: &str) -> Option<&'static Surrogate> {
    let clean = name.trim();
    SURROGATES.iter().find(|s| s.name.eq_ignore_ascii_case(clean))
}

/// Resolves a surrogate name directly into a Data URI for redirection.
pub fn resolve_data_uri(name: &str) -> Option<String> {
    if name == "1x1.gif" {
        return Some(EMPTY_GIF_DATA_URI.to_string());
    }
    lookup(name).map(|s| s.to_data_uri())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_surrogates() {
        assert!(lookup("noopjs").is_some());
        assert!(lookup("google-analytics_analytics.js").is_some());
        assert!(lookup("googletagmanager_gtm.js").is_some());
        assert!(lookup("non-existent-shim").is_none());
    }

    #[test]
    fn test_resolve_data_uri() {
        let uri = resolve_data_uri("google-analytics_analytics.js").unwrap();
        assert!(uri.starts_with("data:application/javascript;utf8,"));
        assert!(uri.contains("window.ga"));

        let gif = resolve_data_uri("1x1.gif").unwrap();
        assert!(gif.starts_with("data:image/gif;base64,"));
    }
}
