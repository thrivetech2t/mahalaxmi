// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//! Regression tests that lock down the public API surface used by the
//! mahalaxmi-detection example programs (examples/detection/01-basic-detection.rs
//! and examples/detection/02-custom-rules.rs).
//!
//! All tests exercise only the crate's public API.  No private modules are
//! imported.

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::ActionType;
use mahalaxmi_detection::{BuiltinRuleSets, DetectionRule, RuleMatcher};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn en_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn generic_matcher() -> RuleMatcher {
    let i18n = en_i18n();
    let rules = BuiltinRuleSets::generic();
    RuleMatcher::new(rules, &i18n).expect("Failed to create RuleMatcher from generic rules")
}

// ---------------------------------------------------------------------------
// BuiltinRuleSets
// ---------------------------------------------------------------------------

#[test]
fn builtin_generic_rules_is_nonempty() {
    let rules = BuiltinRuleSets::generic();
    assert!(!rules.is_empty(), "BuiltinRuleSets::generic() should return at least one rule");
}

#[test]
fn builtin_claude_code_rules_is_nonempty() {
    let rules = BuiltinRuleSets::claude_code();
    assert!(
        !rules.is_empty(),
        "BuiltinRuleSets::claude_code() should return at least one rule"
    );
}

// ---------------------------------------------------------------------------
// RuleMatcher construction
// ---------------------------------------------------------------------------

#[test]
fn rule_matcher_new_with_valid_rules_succeeds() {
    let i18n = en_i18n();
    let rules = BuiltinRuleSets::generic();
    let result = RuleMatcher::new(rules, &i18n);
    assert!(result.is_ok(), "RuleMatcher::new() with valid rules should succeed");
}

// ---------------------------------------------------------------------------
// RuleMatcher::evaluate — no-match cases
// ---------------------------------------------------------------------------

#[test]
fn rule_matcher_evaluate_returns_none_for_blank_line() {
    let mut matcher = generic_matcher();
    let result = matcher.evaluate("  ", None, None);
    assert!(result.is_none(), "Blank line should not match any generic rule");
}

#[test]
fn rule_matcher_evaluate_returns_none_for_normal_output() {
    let mut matcher = generic_matcher();
    let result = matcher.evaluate("Analyzing the codebase...", None, None);
    assert!(
        result.is_none(),
        "Ordinary informational text should not match any generic rule"
    );
}

// ---------------------------------------------------------------------------
// RuleMatcher::evaluate — match cases
// ---------------------------------------------------------------------------

#[test]
fn rule_matcher_evaluate_detects_error_keyword() {
    // The generic ruleset does not include a general "Error:" pattern;
    // we build a custom matcher that does.
    let i18n = en_i18n();
    let rules = vec![DetectionRule::new("error-rule", ActionType::EscalateToManager)
        .with_contains_pattern("Error:")];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let result = matcher.evaluate("Error: cannot find module", None, None);
    assert!(result.is_some(), "Expected a match for 'Error: cannot find module'");
    let result = result.unwrap();
    assert!(result.matched, "DetectionResult.matched should be true");
}

#[test]
fn rule_matcher_evaluate_result_has_rule_name() {
    let i18n = en_i18n();
    let rules = vec![DetectionRule::new("my-rule", ActionType::EscalateToManager)
        .with_contains_pattern("TRIGGER")];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let result = matcher.evaluate("TRIGGER text", None, None).unwrap();
    assert!(
        result.matched_rule_name.is_some(),
        "Matched result should carry the rule name"
    );
    assert_eq!(result.matched_rule_name.unwrap(), "my-rule");
}

#[test]
fn rule_matcher_evaluate_result_has_action() {
    let i18n = en_i18n();
    let rules = vec![
        DetectionRule::new("action-rule", ActionType::CompleteWorkerCycle)
            .with_contains_pattern("DONE"),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let result = matcher.evaluate("Task DONE", None, None).unwrap();
    assert_eq!(result.action, ActionType::CompleteWorkerCycle);
}

#[test]
fn rule_matcher_rule_count_reflects_input() {
    let i18n = en_i18n();
    let generic = BuiltinRuleSets::generic();
    let rule_count_before = generic.len();
    let matcher = RuleMatcher::new(generic, &i18n).unwrap();
    assert!(
        matcher.rule_count() >= rule_count_before,
        "rule_count() should be >= the number of input rules"
    );
}

// ---------------------------------------------------------------------------
// Custom rules — pattern types
// ---------------------------------------------------------------------------

#[test]
fn custom_rule_with_contains_pattern_fires() {
    let i18n = en_i18n();
    let rules = vec![
        DetectionRule::new("prompt-rule", ActionType::ContinueProcessing)
            .with_contains_pattern("READY>"),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let result = matcher.evaluate("READY> waiting", None, None);
    assert!(result.is_some(), "Rule with contains pattern 'READY>' should fire");
}

#[test]
fn custom_rule_with_regex_pattern_fires() {
    let i18n = en_i18n();
    let rules = vec![
        DetectionRule::new("done-rule", ActionType::CompleteWorkerCycle)
            .with_regex_pattern(r"={2,}\s*DONE\s*={2,}"),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let result = matcher.evaluate("== DONE ==", None, None);
    assert!(result.is_some(), "Regex rule should fire on '== DONE =='");
}

#[test]
fn custom_rule_with_regex_fires_not_on_mismatch() {
    let i18n = en_i18n();
    let rules = vec![
        DetectionRule::new("done-rule", ActionType::CompleteWorkerCycle)
            .with_regex_pattern(r"={2,}\s*DONE\s*={2,}"),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    // Only a single '=' on each side — does not satisfy `={2,}`.
    let result = matcher.evaluate("= DONE =", None, None);
    assert!(
        result.is_none(),
        "Regex rule requiring =={{2,}} should not fire on '= DONE ='"
    );
}

// ---------------------------------------------------------------------------
// Custom rules — provider filter
// ---------------------------------------------------------------------------

#[test]
fn custom_rule_with_provider_filter_fires_for_matching_provider() {
    let i18n = en_i18n();
    let rules = vec![
        DetectionRule::new("provider-rule", ActionType::ContinueProcessing)
            .with_contains_pattern("MSG")
            .with_provider_filter(vec!["my-provider".to_string()]),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let result = matcher.evaluate("MSG here", Some("my-provider"), None);
    assert!(
        result.is_some(),
        "Provider-filtered rule should fire when provider matches"
    );
}

#[test]
fn custom_rule_with_provider_filter_suppressed_for_other_provider() {
    let i18n = en_i18n();
    let rules = vec![
        DetectionRule::new("provider-rule", ActionType::ContinueProcessing)
            .with_contains_pattern("MSG")
            .with_provider_filter(vec!["my-provider".to_string()]),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let result = matcher.evaluate("MSG here", Some("other-provider"), None);
    assert!(
        result.is_none(),
        "Provider-filtered rule should not fire for a different provider"
    );
}

// ---------------------------------------------------------------------------
// Custom rules — priority ordering
// ---------------------------------------------------------------------------

#[test]
fn custom_rule_priority_higher_priority_wins() {
    // Lower number = higher priority (fires first).
    let i18n = en_i18n();
    let rules = vec![
        DetectionRule::new("high-priority", ActionType::CompleteWorkerCycle)
            .with_contains_pattern("MATCH")
            .with_priority(10),
        DetectionRule::new("low-priority", ActionType::EscalateToManager)
            .with_contains_pattern("MATCH")
            .with_priority(90),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let result = matcher.evaluate("MATCH line", None, None).unwrap();
    // The rule with priority 10 should win.
    assert_eq!(
        result.matched_rule_name.as_deref(),
        Some("high-priority"),
        "Lower priority number should win"
    );
    assert_eq!(result.action, ActionType::CompleteWorkerCycle);
}

// ---------------------------------------------------------------------------
// Custom rules — cooldown
// ---------------------------------------------------------------------------

#[test]
fn custom_rule_with_cooldown_suppresses_second_match() {
    let i18n = en_i18n();
    // 5-second cooldown — both evaluations happen within the same millisecond.
    let rules = vec![
        DetectionRule::new("cool-rule", ActionType::ContinueProcessing)
            .with_contains_pattern("HIT")
            .with_cooldown_ms(5_000),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();
    let first = matcher.evaluate("HIT text", None, None);
    assert!(first.is_some(), "First evaluation should match");
    let second = matcher.evaluate("HIT text", None, None);
    assert!(
        second.is_none(),
        "Second evaluation within cooldown window should not match"
    );
}

// ---------------------------------------------------------------------------
// DetectionRule builder chaining
// ---------------------------------------------------------------------------

#[test]
fn detection_rule_builder_chaining() {
    // Verify that all builder methods can be chained without panicking.
    let _rule = DetectionRule::new("r1", ActionType::ContinueProcessing)
        .with_contains_pattern("X")
        .with_priority(5)
        .with_cooldown_ms(1_000);
}

// ---------------------------------------------------------------------------
// Worker cycle simulation
// ---------------------------------------------------------------------------

#[test]
fn complete_worker_cycle_simulation() {
    let i18n = en_i18n();
    let rules = vec![
        // Informational — should not fire.
        DetectionRule::new("info-rule", ActionType::ContinueProcessing)
            .with_contains_pattern("THIS_WILL_NEVER_MATCH_NORMAL_OUTPUT")
            .with_priority(50),
        // Error — should fire on "Error: ..." lines.
        DetectionRule::new("error-rule", ActionType::EscalateToManager)
            .with_contains_pattern("Error:")
            .with_priority(20),
        // Shell prompt — signals cycle completion.
        DetectionRule::new("prompt-rule", ActionType::CompleteWorkerCycle)
            .with_regex_pattern(r"\$\s*$")
            .with_priority(90),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();

    // Normal informational line — no match expected.
    let result = matcher.evaluate("Analyzing the codebase...", None, None);
    assert!(result.is_none(), "Informational line should not match");

    // Error line — match expected, action = EscalateToManager.
    let result = matcher.evaluate("Error: missing dependency", None, None);
    assert!(result.is_some(), "Error line should match");
    assert_eq!(result.unwrap().action, ActionType::EscalateToManager);

    // Shell prompt — match expected, action = CompleteWorkerCycle.
    let result = matcher.evaluate("user@host:~/project$ ", None, None);
    assert!(result.is_some(), "Shell prompt should match");
    assert_eq!(result.unwrap().action, ActionType::CompleteWorkerCycle);
}

// ---------------------------------------------------------------------------
// Combining builtin and custom rules
// ---------------------------------------------------------------------------

#[test]
fn builtin_rules_combined_with_custom_rules() {
    let i18n = en_i18n();
    let generic_rules = BuiltinRuleSets::generic();
    let claude_rules = BuiltinRuleSets::claude_code();
    let custom_rules = vec![
        DetectionRule::new("custom-1", ActionType::ContinueProcessing)
            .with_contains_pattern("CUSTOM_A"),
        DetectionRule::new("custom-2", ActionType::ContinueProcessing)
            .with_contains_pattern("CUSTOM_B"),
    ];

    let total_input = generic_rules.len() + claude_rules.len() + custom_rules.len();

    let mut all_rules = BuiltinRuleSets::generic();
    all_rules.extend(BuiltinRuleSets::claude_code());
    all_rules.extend(custom_rules);

    let matcher = RuleMatcher::new(all_rules, &i18n).unwrap();
    assert!(
        matcher.rule_count() >= total_input,
        "rule_count() should be at least the sum of all input rule sets"
    );
}

// ---------------------------------------------------------------------------
// Cooldown reset
// ---------------------------------------------------------------------------

#[test]
fn reset_cooldowns_allows_rematch() {
    let i18n = en_i18n();
    let rules = vec![
        DetectionRule::new("cool-rule", ActionType::ContinueProcessing)
            .with_contains_pattern("HIT")
            .with_cooldown_ms(5_000),
    ];
    let mut matcher = RuleMatcher::new(rules, &i18n).unwrap();

    // First match succeeds.
    let first = matcher.evaluate("HIT text", None, None);
    assert!(first.is_some(), "First evaluation should match");

    // Within cooldown — second match is suppressed.
    let second = matcher.evaluate("HIT text", None, None);
    assert!(second.is_none(), "Second evaluation should be suppressed by cooldown");

    // After reset, the rule should fire again.
    matcher.reset_cooldowns();
    let third = matcher.evaluate("HIT text", None, None);
    assert!(
        third.is_some(),
        "After reset_cooldowns(), the rule should fire again"
    );
}
