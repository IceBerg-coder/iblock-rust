import assert from 'node:assert';
import fs from 'node:fs';
import init, { WasmEngine, RequestTypeFlags } from '../dist/wasm/iblock_wasm.js';

console.log('Testing WasmEngine (Web/ESM Target) in Node.js...');

// Initialize Wasm module with bytes
const wasmBytes = fs.readFileSync(new URL('../dist/wasm/iblock_wasm_bg.wasm', import.meta.url));
await init({ module_or_path: wasmBytes });

// 1. Initialize engine
const engine = new WasmEngine();
assert.strictEqual(engine.ruleCount(), 0);

// 2. Load filter list
const filterList = `
! Sample Filter List
||doubleclick.net^
||google-analytics.com/analytics.js$script,redirect=google-analytics_analytics.js
||cdn.adservice.com^$third-party,script
@@||allowed.tracker.org^
##.generic-ad
example.com##.sponsor-banner
example.com#@#.generic-ad
example.com##div:has(> .sponsored-badge)
`;

const count = engine.loadList(filterList);
console.log(`Loaded ${count} rules into WasmEngine`);
assert.strictEqual(count, 8);
assert.strictEqual(engine.ruleCount(), 8);
assert.strictEqual(engine.cosmeticRuleCount(), 4);

// 3. Match requests
const scriptType = RequestTypeFlags.script;
const thirdPartyFlag = RequestTypeFlags.thirdParty;

// Block doubleclick.net
const r1 = engine.matchRequest('https://doubleclick.net/ad.js', 'example.com', scriptType);
assert.strictEqual(r1.isBlocked(), true);
assert.strictEqual(r1.action, 1);
console.log('✔ doubleclick.net correctly blocked');

// Redirect google-analytics
const r2 = engine.matchRequest('https://google-analytics.com/analytics.js', 'example.com', scriptType);
assert.strictEqual(r2.isRedirected(), true);
assert.strictEqual(r2.action, 3);
assert.strictEqual(r2.redirectTarget, 'google-analytics_analytics.js');
assert(r2.redirectDataUri.startsWith('data:application/javascript'));
console.log('✔ google-analytics redirected to Data URI:', r2.redirectDataUri.slice(0, 45) + '...');

// Whitelisted allowed.tracker.org
const r3 = engine.matchRequest('https://allowed.tracker.org/script.js', 'example.com', scriptType);
assert.strictEqual(r3.isAllowed(), true);
assert.strictEqual(r3.action, 2);
console.log('✔ allowed.tracker.org correctly whitelisted');

// Non-blocked wikipedia
const r4 = engine.matchRequest('https://en.wikipedia.org/wiki/Main_Page', 'wikipedia.org', 0);
assert.strictEqual(r4.isBlocked(), false);
assert.strictEqual(r4.isAllowed(), false);
assert.strictEqual(r4.action, 0);
console.log('✔ wikipedia.org correctly allowed (no match)');

// 4. Test Cosmetic & Procedural evaluation
console.log('Testing Cosmetic & Procedural filtering in Wasm...');
const cosmeticsExample = engine.retrieveCosmetics('example.com');
assert.strictEqual(cosmeticsExample.hasSelectors(), true);
// .sponsor-banner should be present, .generic-ad should be unhidden by #@#
assert(cosmeticsExample.stylesheet.includes('.sponsor-banner'));
assert(!cosmeticsExample.stylesheet.includes('.generic-ad'));
console.log('✔ example.com CSS stylesheet correctly includes specific rule and applies exception');

const procSels = cosmeticsExample.proceduralSelectors;
assert.strictEqual(procSels.length, 1);
assert.strictEqual(procSels[0], 'div:has(> .sponsored-badge)');
console.log('✔ example.com procedural selector correctly dispatched:', procSels[0]);

const cosmeticsOther = engine.retrieveCosmetics('other.org');
assert(cosmeticsOther.stylesheet.includes('.generic-ad'));
assert(!cosmeticsOther.stylesheet.includes('.sponsor-banner'));
console.log('✔ other.org CSS stylesheet correctly inherits generic ad rule');

// 5. Test Dynamic Matrix Firewall
console.log('Testing Dynamic Matrix Firewall in Wasm...');
engine.addDynamicRule('* * 3p-script block');
engine.addDynamicRule('myblog.com cdn.trusted.org 3p-script allow');
assert.strictEqual(engine.dynamicRuleCount(), 2);

// 3p script from untrusted origin on myblog -> blocked by dynamic rule
const dBlock = engine.matchRequest('https://untrusted.org/tracker.js', 'myblog.com', scriptType | thirdPartyFlag);
assert.strictEqual(dBlock.isBlocked(), true);
console.log('✔ Dynamic Firewall globally blocked 3p script');

// 3p script from trusted origin on myblog -> allowed by local dynamic rule
const dAllow = engine.matchRequest('https://cdn.trusted.org/app.js', 'myblog.com', scriptType | thirdPartyFlag);
assert.strictEqual(dAllow.isAllowed(), true);
console.log('✔ Dynamic Firewall locally allowed 3p script override');

// 6. Test LZ4 serialization & hydration
console.log('Testing LZ4 binary serialization to Uint8Array...');
const compressedBytes = engine.toCompressedBytes();
assert(compressedBytes instanceof Uint8Array);
console.log(`Serialized compressed Uint8Array size: ${compressedBytes.length} bytes`);

// 7. Restore from compressed bytes
const restoredEngine = WasmEngine.fromCompressedBytes(compressedBytes);
assert.strictEqual(restoredEngine.ruleCount(), 8);
assert.strictEqual(restoredEngine.cosmeticRuleCount(), 4);
assert.strictEqual(restoredEngine.dynamicRuleCount(), 2);

const rRestoredDynamic = restoredEngine.matchRequest('https://untrusted.org/tracker.js', 'myblog.com', scriptType | thirdPartyFlag);
assert.strictEqual(rRestoredDynamic.isBlocked(), true);

const rRestored = restoredEngine.matchRequest('https://doubleclick.net/ad.js', 'example.com', scriptType);
assert.strictEqual(rRestored.isBlocked(), true);

const restoredCosmetics = restoredEngine.retrieveCosmetics('example.com');
assert(restoredCosmetics.stylesheet.includes('.sponsor-banner'));
assert(!restoredCosmetics.stylesheet.includes('.generic-ad'));
console.log('✔ Rehydrated WasmEngine verified network and cosmetic rules identically in JS!');

console.log('\nAll Wasm integration tests passed successfully!');
