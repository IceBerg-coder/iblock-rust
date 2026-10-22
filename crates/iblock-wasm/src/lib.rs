//! WebAssembly bindings for iBlock content filtering engine.

use wasm_bindgen::prelude::*;
use iblock_core::{MatchResult, RequestTypes, StaticNetFilteringEngine};

/// Result of evaluating an HTTP request.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WasmMatchResult {
    action: u8,
    redirect: Option<String>,
}

#[wasm_bindgen]
impl WasmMatchResult {
    /// 0 = NoMatch, 1 = Block, 2 = Allow, 3 = Redirect
    #[wasm_bindgen(getter)]
    pub fn action(&self) -> u8 {
        self.action
    }

    #[wasm_bindgen(getter, js_name = redirectTarget)]
    pub fn redirect_target(&self) -> Option<String> {
        self.redirect.clone()
    }

    /// Resolves the redirection target into a ready-to-use Data URI.
    #[wasm_bindgen(getter, js_name = redirectDataUri)]
    pub fn redirect_data_uri(&self) -> Option<String> {
        self.redirect.as_deref().and_then(iblock_core::surrogate::resolve_data_uri)
    }

    #[wasm_bindgen(js_name = isBlocked)]
    pub fn is_blocked(&self) -> bool {
        self.action == 1
    }

    #[wasm_bindgen(js_name = isAllowed)]
    pub fn is_allowed(&self) -> bool {
        self.action == 2
    }

    #[wasm_bindgen(js_name = isRedirected)]
    pub fn is_redirected(&self) -> bool {
        self.action == 3
    }
}

/// Result of evaluating cosmetic selectors for a webpage.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WasmCosmeticResult {
    stylesheet: String,
    procedural_selectors: Vec<String>,
}

#[wasm_bindgen]
impl WasmCosmeticResult {
    #[wasm_bindgen(getter)]
    pub fn stylesheet(&self) -> String {
        self.stylesheet.clone()
    }

    #[wasm_bindgen(getter, js_name = proceduralSelectors)]
    pub fn procedural_selectors(&self) -> Vec<String> {
        self.procedural_selectors.clone()
    }

    #[wasm_bindgen(js_name = hasSelectors)]
    pub fn has_selectors(&self) -> bool {
        !self.stylesheet.is_empty() || !self.procedural_selectors.is_empty()
    }
}

/// WebAssembly-exported Static Network Filtering Engine.
#[wasm_bindgen]
pub struct WasmEngine {
    inner: StaticNetFilteringEngine,
}

#[wasm_bindgen]
impl WasmEngine {
    /// Creates a new, empty WasmEngine instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEngine {
        WasmEngine {
            inner: StaticNetFilteringEngine::new(),
        }
    }

    /// Ingests a text filter list (e.g. EasyList) and returns the number of loaded rules.
    #[wasm_bindgen(js_name = loadList)]
    pub fn load_list(&mut self, content: &str) -> u32 {
        self.inner.load_list(content) as u32
    }

    /// Evaluates an HTTP network request against all loaded rules in sub-microsecond time.
    ///
    /// - `url`: Destination URL (e.g. "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js")
    /// - `source_domain`: Originating page domain (e.g. "example.com")
    /// - `request_type`: Bitfield of RequestTypes (e.g. 1 for SCRIPT)
    #[wasm_bindgen(js_name = matchRequest)]
    pub fn match_request(&self, url: &str, source_domain: &str, request_type: u32) -> WasmMatchResult {
        let types = RequestTypes::from_bits_truncate(request_type);
        let res = self.inner.match_request(url, source_domain, types);
        match res {
            MatchResult::NoMatch => WasmMatchResult {
                action: 0,
                redirect: None,
            },
            MatchResult::Block => WasmMatchResult {
                action: 1,
                redirect: None,
            },
            MatchResult::Allow => WasmMatchResult {
                action: 2,
                redirect: None,
            },
            MatchResult::Redirect(shim) => WasmMatchResult {
                action: 3,
                redirect: Some(shim),
            },
        }
    }

    /// Serializes and compresses the entire compiled rule database into an LZ4 binary buffer.
    #[wasm_bindgen(js_name = toCompressedBytes)]
    pub fn to_compressed_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.inner
            .to_compressed_bytes()
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Hydrates a complete WasmEngine instantaneously from an LZ4-compressed binary buffer.
    #[wasm_bindgen(js_name = fromCompressedBytes)]
    pub fn from_compressed_bytes(bytes: &[u8]) -> Result<WasmEngine, JsValue> {
        let inner = StaticNetFilteringEngine::from_compressed_bytes(bytes)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        Ok(WasmEngine { inner })
    }

    /// Total number of active rules compiled in the engine.
    #[wasm_bindgen(js_name = ruleCount)]
    pub fn rule_count(&self) -> u32 {
        self.inner.rule_count() as u32
    }

    /// Evaluates all cosmetic CSS and procedural hiding rules for a given webpage hostname.
    #[wasm_bindgen(js_name = retrieveCosmetics)]
    pub fn retrieve_cosmetics(&self, hostname: &str) -> WasmCosmeticResult {
        let res = self.inner.retrieve_cosmetics(hostname);
        WasmCosmeticResult {
            stylesheet: res.stylesheet,
            procedural_selectors: res.procedural_selectors,
        }
    }

    /// Number of cosmetic rules currently active.
    #[wasm_bindgen(js_name = cosmeticRuleCount)]
    pub fn cosmetic_rule_count(&self) -> u32 {
        self.inner.cosmetic_rule_count() as u32
    }

    /// Configures a dynamic matrix firewall rule (e.g. `* * 3p-script block`).
    #[wasm_bindgen(js_name = addDynamicRule)]
    pub fn add_dynamic_rule(&mut self, rule_line: &str) -> bool {
        self.inner.add_dynamic_rule_line(rule_line)
    }

    /// Number of dynamic matrix firewall rules configured.
    #[wasm_bindgen(js_name = dynamicRuleCount)]
    pub fn dynamic_rule_count(&self) -> u32 {
        self.inner.dynamic_rule_count() as u32
    }
}

/// Request type flag constants exposed to JavaScript.
#[wasm_bindgen]
pub struct RequestTypeFlags;

#[wasm_bindgen]
impl RequestTypeFlags {
    #[wasm_bindgen(getter)]
    pub fn script() -> u32 {
        RequestTypes::SCRIPT.bits()
    }

    #[wasm_bindgen(getter)]
    pub fn image() -> u32 {
        RequestTypes::IMAGE.bits()
    }

    #[wasm_bindgen(getter)]
    pub fn stylesheet() -> u32 {
        RequestTypes::STYLESHEET.bits()
    }

    #[wasm_bindgen(getter)]
    pub fn xhr() -> u32 {
        RequestTypes::XMLHTTPREQUEST.bits()
    }

    #[wasm_bindgen(getter)]
    pub fn subdocument() -> u32 {
        RequestTypes::SUBDOCUMENT.bits()
    }

    #[wasm_bindgen(getter, js_name = firstParty)]
    pub fn first_party() -> u32 {
        RequestTypes::FIRST_PARTY.bits()
    }

    #[wasm_bindgen(getter, js_name = thirdParty)]
    pub fn third_party() -> u32 {
        RequestTypes::THIRD_PARTY.bits()
    }
}
