use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::time::Instant;

use iblock_core::{MatchResult, RequestTypes, StaticNetFilteringEngine};

fn print_usage() {
    println!("iBlock Rust Engine CLI");
    println!("Usage:");
    println!("  iblock-cli audit <filter_list_file> <url> [source_domain]");
    println!("  iblock-cli bench <filter_list_file>");
    println!("  iblock-cli compile <filter_list_file> <output_cache_file>");
    println!("  iblock-cli load-bench <cache_file>");
    println!("  iblock-cli repl <filter_list_file>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "audit" => {
            if args.len() < 4 {
                println!("Error: Missing arguments for audit");
                print_usage();
                return;
            }
            let list_path = &args[2];
            let url = &args[3];
            let source_domain = args.get(4).map(|s| s.as_str()).unwrap_or("example.com");

            let mut engine = StaticNetFilteringEngine::new();
            let content = fs::read_to_string(list_path).expect("Failed to read filter list file");
            let count = engine.load_list(&content);
            println!("Loaded {} rules from {}", count, list_path);

            let start = Instant::now();
            let result = engine.match_request(url, source_domain, RequestTypes::SCRIPT | RequestTypes::THIRD_PARTY);
            let elapsed = start.elapsed();

            println!("Matching URL: {}", url);
            println!("Source Domain: {}", source_domain);
            println!("Decision: {:?}", result);
            println!("Evaluation time: {:?}", elapsed);
        }
        "bench" => {
            if args.len() < 3 {
                println!("Error: Missing filter list file path");
                return;
            }
            let list_path = &args[2];
            let content = fs::read_to_string(list_path).expect("Failed to read filter list file");

            println!("Benchmarking list loading: {}", list_path);
            let t0 = Instant::now();
            let mut engine = StaticNetFilteringEngine::new();
            let count = engine.load_list(&content);
            let load_elapsed = t0.elapsed();
            println!("Loaded {} rules in {:?}", count, load_elapsed);

            let test_urls = [
                "https://doubleclick.net/pagead/ads?client=ca-pub-123",
                "https://google-analytics.com/analytics.js",
                "https://facebook.com/tr?id=123",
                "https://secure.adnxs.com/seg?add=1",
                "https://wikipedia.org/wiki/Rust_(programming_language)",
            ];

            println!("\nRunning 100,000 lookup iterations...");
            let t1 = Instant::now();
            let mut matches = 0;
            for i in 0..100_000 {
                let url = test_urls[i % test_urls.len()];
                if engine.match_request(url, "example.com", RequestTypes::SCRIPT) != MatchResult::NoMatch {
                    matches += 1;
                }
            }
            let match_elapsed = t1.elapsed();
            println!("100,000 matches finished in {:?}", match_elapsed);
            println!("Average latency per request: {:.2} ns", (match_elapsed.as_nanos() as f64) / 100_000.0);
            println!("Blocked/Matched: {}/100,000", matches);
        }
        "compile" => {
            if args.len() < 4 {
                println!("Error: Missing arguments for compile (input_list, output_cache)");
                return;
            }
            let input_path = &args[2];
            let output_path = &args[3];

            println!("Reading list: {}", input_path);
            let content = fs::read_to_string(input_path).expect("Failed to read filter list");
            let mut engine = StaticNetFilteringEngine::new();
            let count = engine.load_list(&content);
            println!("Compiled {} rules. Writing compressed binary cache to: {}", count, output_path);

            let t0 = Instant::now();
            engine.save_to_file(output_path).expect("Failed to write cache file");
            println!("Cache written in {:?}", t0.elapsed());
            let meta = fs::metadata(output_path).expect("Failed to get cache file size");
            println!("Output cache size: {} bytes ({:.2} KB)", meta.len(), (meta.len() as f64) / 1024.0);
        }
        "load-bench" => {
            if args.len() < 3 {
                println!("Error: Missing cache file path");
                return;
            }
            let cache_path = &args[2];
            println!("Hydrating engine from binary cache: {}", cache_path);
            let t0 = Instant::now();
            let engine = StaticNetFilteringEngine::load_from_file(cache_path).expect("Failed to load cache");
            let elapsed = t0.elapsed();
            println!("Hydrated {} rules in {:?}", engine.rule_count(), elapsed);

            let res = engine.match_request("https://doubleclick.net/ad.js", "example.com", RequestTypes::SCRIPT);
            println!("Verification match (doubleclick.net): {:?}", res);
        }
        "repl" => {
            if args.len() < 3 {
                println!("Error: Missing filter list file path");
                return;
            }
            let list_path = &args[2];
            let content = fs::read_to_string(list_path).expect("Failed to read filter list file");
            let mut engine = StaticNetFilteringEngine::new();
            let count = engine.load_list(&content);
            println!("Loaded {} rules. Enter URLs to test (type 'exit' to quit):", count);

            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line.unwrap();
                let trimmed = line.trim();
                if trimmed == "exit" || trimmed == "quit" {
                    break;
                }
                if trimmed.is_empty() {
                    continue;
                }
                let res = engine.match_request(trimmed, "example.com", RequestTypes::SCRIPT | RequestTypes::THIRD_PARTY);
                println!("-> {:?}", res);
            }
        }
        _ => {
            print_usage();
        }
    }
}
