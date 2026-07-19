use super::*;

fn names(matches: &[FunctionMatch]) -> Vec<&str> {
    matches.iter().map(|m| m.name.as_str()).collect()
}

#[test]
fn monthly_loan_payment_ranks_pmt_first() {
    let registry = Registry::new();
    let results = search_functions(&registry, "monthly loan payment", 5);
    assert_eq!(results.first().map(|m| m.name.as_str()), Some("PMT"), "got: {:?}", names(&results));
}

#[test]
fn results_carry_signature_and_metadata() {
    let registry = Registry::new();
    let results = search_functions(&registry, "monthly loan payment", 3);
    let top = &results[0];
    assert_eq!(top.name, "PMT");
    assert_eq!(top.signature, "PMT(rate, nper, pv)");
    assert_eq!(top.category, "financial");
    assert!(!top.description.is_empty());
    assert!(top.score > 0);
}

#[test]
fn results_ranked_descending_by_score() {
    let registry = Registry::new();
    let results = search_functions(&registry, "loan payment interest", 20);
    for pair in results.windows(2) {
        assert!(pair[0].score >= pair[1].score, "not descending: {:?}", names(&results));
    }
}

#[test]
fn deterministic_identical_query_identical_ranking() {
    let registry = Registry::new();
    let a = search_functions(&registry, "present value of an investment", 10);
    let b = search_functions(&registry, "present value of an investment", 10);
    assert_eq!(a, b);
}

#[test]
fn ties_broken_by_name_ascending() {
    let registry = Registry::new();
    let results = search_functions(&registry, "loan payment interest", 30);
    for pair in results.windows(2) {
        if pair[0].score == pair[1].score {
            assert!(pair[0].name < pair[1].name, "tie not name-ordered: {:?}", names(&results));
        }
    }
}

#[test]
fn limit_caps_result_count() {
    let registry = Registry::new();
    let results = search_functions(&registry, "value", 3);
    assert!(results.len() <= 3);
}

#[test]
fn limit_zero_returns_all_matches() {
    let registry = Registry::new();
    let capped = search_functions(&registry, "date", 2);
    let uncapped = search_functions(&registry, "date", 0);
    assert!(uncapped.len() >= capped.len());
    assert!(uncapped.len() > 2);
}

#[test]
fn exact_name_query_ranks_that_function_first() {
    let registry = Registry::new();
    let results = search_functions(&registry, "PMT", 5);
    assert_eq!(results.first().map(|m| m.name.as_str()), Some("PMT"));
}

#[test]
fn empty_query_returns_no_matches() {
    let registry = Registry::new();
    assert!(search_functions(&registry, "", 10).is_empty());
    assert!(search_functions(&registry, "   the of and", 10).is_empty());
}

#[test]
fn nonsense_query_returns_no_matches() {
    let registry = Registry::new();
    assert!(search_functions(&registry, "zxqwvfoobarbaz", 10).is_empty());
}

#[test]
fn synonym_bridges_average_to_mean() {
    // AVERAGE's description mentions "mean"; a query for "mean" should surface it.
    let registry = Registry::new();
    let results = search_functions(&registry, "mean", 10);
    assert!(names(&results).contains(&"AVERAGE"), "got: {:?}", names(&results));
}

#[test]
fn matches_are_only_user_facing_functions() {
    // Every returned name must resolve in the registry (no alias/internal leakage).
    let registry = Registry::new();
    let results = search_functions(&registry, "value", 50);
    for m in &results {
        assert!(registry.get(&m.name).is_some(), "unresolvable: {}", m.name);
    }
}
