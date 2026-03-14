// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//! API regression tests for the indexing crate public surface used by
//! `examples/indexing/01-parse-repository.rs` and `02-symbol-extraction.rs`.
//!
//! Each test targets a single public API call so that a rename, signature
//! change, or deleted method causes an obvious compile failure here before
//! it breaks the examples.

use mahalaxmi_indexing::{
    ExtractorFactory, FileDependency, FileDependencyGraph, GroupBy, LanguageRegistry, RepoMapConfig,
    SupportedLanguage, SymbolKind,
};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use std::path::Path;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ---------------------------------------------------------------------------
// SupportedLanguage::from_extension
// ---------------------------------------------------------------------------

#[test]
fn from_extension_rs_is_rust() {
    assert_eq!(SupportedLanguage::from_extension(".rs"), Some(SupportedLanguage::Rust));
}

#[test]
fn from_extension_ts_is_typescript() {
    assert_eq!(SupportedLanguage::from_extension(".ts"), Some(SupportedLanguage::TypeScript));
}

#[test]
fn from_extension_tsx_is_typescript() {
    assert_eq!(SupportedLanguage::from_extension(".tsx"), Some(SupportedLanguage::TypeScript));
}

#[test]
fn from_extension_py_is_python() {
    assert_eq!(SupportedLanguage::from_extension(".py"), Some(SupportedLanguage::Python));
}

#[test]
fn from_extension_go_is_go() {
    assert_eq!(SupportedLanguage::from_extension(".go"), Some(SupportedLanguage::Go));
}

#[test]
fn from_extension_java_is_java() {
    assert_eq!(SupportedLanguage::from_extension(".java"), Some(SupportedLanguage::Java));
}

#[test]
fn from_extension_c_is_c() {
    assert_eq!(SupportedLanguage::from_extension(".c"), Some(SupportedLanguage::C));
}

#[test]
fn from_extension_cpp_is_cpp() {
    assert_eq!(SupportedLanguage::from_extension(".cpp"), Some(SupportedLanguage::Cpp));
}

#[test]
fn from_extension_cc_is_cpp() {
    assert_eq!(SupportedLanguage::from_extension(".cc"), Some(SupportedLanguage::Cpp));
}

#[test]
fn from_extension_unknown_is_none() {
    assert_eq!(SupportedLanguage::from_extension(".unknown"), None);
}

#[test]
fn from_extension_just_dot_is_none() {
    assert_eq!(SupportedLanguage::from_extension("."), None);
}

#[test]
fn from_extension_empty_is_none() {
    assert_eq!(SupportedLanguage::from_extension(""), None);
}

// ---------------------------------------------------------------------------
// SupportedLanguage methods
// ---------------------------------------------------------------------------

#[test]
fn rust_as_str_is_non_empty() {
    assert!(!SupportedLanguage::Rust.as_str().is_empty());
}

#[test]
fn rust_extensions_contains_rs() {
    assert!(SupportedLanguage::Rust.extensions().contains(&".rs"));
}

// ---------------------------------------------------------------------------
// LanguageRegistry
// ---------------------------------------------------------------------------

#[test]
fn with_defaults_language_count_at_least_6() {
    assert!(LanguageRegistry::with_defaults().language_count() >= 6);
}

#[test]
fn with_defaults_supports_rust() {
    assert!(LanguageRegistry::with_defaults().is_supported(&SupportedLanguage::Rust));
}

#[test]
fn with_defaults_get_rust_is_some() {
    assert!(LanguageRegistry::with_defaults().get(&SupportedLanguage::Rust).is_some());
}

#[test]
fn with_defaults_supported_languages_at_least_6() {
    assert!(LanguageRegistry::with_defaults().supported_languages().len() >= 6);
}

// ---------------------------------------------------------------------------
// ExtractorFactory
// ---------------------------------------------------------------------------

const RUST_SOURCE: &str = r#"
pub fn greet(name: &str) -> String { format!("Hello, {}!", name) }
pub struct Config { pub host: String }
"#;

const RUST_WITH_USE: &str = r#"
use std::collections::HashMap;
pub fn greet(name: &str) -> String { format!("Hello, {}!", name) }
"#;

#[test]
fn extractor_factory_create_rust_returns_ok() {
    let registry = LanguageRegistry::with_defaults();
    let i18n = i18n();
    let result = ExtractorFactory::create(SupportedLanguage::Rust, &registry, &i18n);
    assert!(result.is_ok(), "ExtractorFactory::create should succeed for Rust");
}

#[test]
fn extractor_language_returns_rust() {
    let registry = LanguageRegistry::with_defaults();
    let i18n = i18n();
    let extractor = ExtractorFactory::create(SupportedLanguage::Rust, &registry, &i18n)
        .expect("extractor creation");
    assert_eq!(extractor.language(), SupportedLanguage::Rust);
}

#[test]
fn extract_symbols_returns_ok_with_symbols() {
    let registry = LanguageRegistry::with_defaults();
    let i18n = i18n();
    let extractor = ExtractorFactory::create(SupportedLanguage::Rust, &registry, &i18n)
        .expect("extractor creation");
    let symbols = extractor
        .extract_symbols(RUST_SOURCE, Path::new("src/main.rs"), &i18n)
        .expect("extract_symbols");
    assert!(!symbols.is_empty(), "should extract at least one symbol");
}

#[test]
fn extracted_symbols_have_non_empty_names() {
    let registry = LanguageRegistry::with_defaults();
    let i18n = i18n();
    let extractor = ExtractorFactory::create(SupportedLanguage::Rust, &registry, &i18n)
        .expect("extractor creation");
    let symbols = extractor
        .extract_symbols(RUST_SOURCE, Path::new("src/main.rs"), &i18n)
        .expect("extract_symbols");
    for sym in &symbols {
        assert!(!sym.name.is_empty(), "symbol name should not be empty");
    }
}

#[test]
fn extracted_symbols_contain_function_kind() {
    let registry = LanguageRegistry::with_defaults();
    let i18n = i18n();
    let extractor = ExtractorFactory::create(SupportedLanguage::Rust, &registry, &i18n)
        .expect("extractor creation");
    let symbols = extractor
        .extract_symbols(RUST_SOURCE, Path::new("src/main.rs"), &i18n)
        .expect("extract_symbols");
    let has_function = symbols.iter().any(|s| s.kind == SymbolKind::Function);
    assert!(has_function, "should detect at least one Function symbol");
}

#[test]
fn extract_imports_returns_ok() {
    let registry = LanguageRegistry::with_defaults();
    let i18n = i18n();
    let extractor = ExtractorFactory::create(SupportedLanguage::Rust, &registry, &i18n)
        .expect("extractor creation");
    let result = extractor.extract_imports(RUST_WITH_USE, Path::new("src/lib.rs"), &i18n);
    assert!(result.is_ok(), "extract_imports should return Ok");
}

// ---------------------------------------------------------------------------
// FileDependencyGraph
// ---------------------------------------------------------------------------

#[test]
fn new_graph_has_file_count_zero() {
    let graph = FileDependencyGraph::new();
    assert_eq!(graph.file_count(), 0);
}

#[test]
fn add_dependency_adds_edge() {
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("a.rs", "b.rs", "crate::b"));
    assert_eq!(graph.dependencies_of(Path::new("a.rs")).len(), 1);
}

#[test]
fn dependencies_of_a_contains_b() {
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("a.rs", "b.rs", "crate::b"));
    assert_eq!(graph.dependencies_of(Path::new("a.rs")).len(), 1);
}

#[test]
fn dependents_of_b_contains_a() {
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("a.rs", "b.rs", "crate::b"));
    assert_eq!(graph.dependents_of(Path::new("b.rs")).len(), 1);
}

#[test]
fn dependents_of_a_is_empty_when_a_only_depends_on_b() {
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("a.rs", "b.rs", "crate::b"));
    assert!(graph.dependents_of(Path::new("a.rs")).is_empty());
}

#[test]
fn bfs_distance_direct_dependency_is_one() {
    use std::path::PathBuf;
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("a.rs", "b.rs", "crate::b"));
    let sources = vec![PathBuf::from("a.rs")];
    let dist = graph.bfs_distance(Path::new("b.rs"), &sources, 4);
    assert_eq!(dist, Some(1));
}

#[test]
fn bfs_distance_target_in_sources_is_zero() {
    use std::path::PathBuf;
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("a.rs", "b.rs", "crate::b"));
    let sources = vec![PathBuf::from("a.rs")];
    let dist = graph.bfs_distance(Path::new("a.rs"), &sources, 4);
    assert_eq!(dist, Some(0));
}

#[test]
fn bfs_distance_unreachable_is_none() {
    use std::path::PathBuf;
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("a.rs", "b.rs", "crate::b"));
    let sources = vec![PathBuf::from("a.rs")];
    let dist = graph.bfs_distance(Path::new("c.rs"), &sources, 4);
    assert_eq!(dist, None);
}

// ---------------------------------------------------------------------------
// RepoMapConfig
// ---------------------------------------------------------------------------

#[test]
fn repo_map_config_default_max_tokens_positive() {
    assert!(RepoMapConfig::default().max_tokens > 0);
}

#[test]
fn group_by_file_and_kind_variants_exist() {
    let file = GroupBy::File;
    let kind = GroupBy::Kind;
    match file {
        GroupBy::File => {}
        GroupBy::Kind => panic!("should be File"),
    }
    match kind {
        GroupBy::Kind => {}
        GroupBy::File => panic!("should be Kind"),
    }
}
