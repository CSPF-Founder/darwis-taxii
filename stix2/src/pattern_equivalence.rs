//! Pattern Semantic Equivalence
//!
//! This module provides utilities for determining semantic equivalence
//! between STIX patterns. Two patterns are semantically equivalent if they
//! would match the same set of observations.
//!
//! The equivalence checking works by:
//! 1. Parsing patterns into ASTs
//! 2. Normalizing the ASTs (ordering, deduplication, simplification)
//! 3. Converting to DNF (Disjunctive Normal Form)
//! 4. Applying special value canonicalization (IPv4/IPv6 CIDR, Windows registry)
//! 5. Comparing the normalized forms

use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::core::error::Result;
use crate::patterns::{
    ComparisonExpression, ComparisonOperator, PatternExpression, PatternValue, parse_pattern,
};

// ============================================================================
// Special Value Canonicalization
// ============================================================================

mod specials {
    /// Canonicalize an IPv4 address value.
    ///
    /// Normalizes CIDR notation by applying the mask to the address.
    /// E.g., "192.168.1.100/24" -> "192.168.1.0/24"
    pub fn canonicalize_ipv4(value: &str) -> String {
        if let Some((addr, mask_str)) = value.split_once('/')
            && let Ok(mask) = mask_str.parse::<u8>()
        {
            if mask >= 32 {
                // /32 is a single host, remove the suffix
                return addr.to_string();
            }
            if let Ok(ip) = addr.parse::<std::net::Ipv4Addr>() {
                let bits = u32::from(ip);
                let mask_bits = if mask == 0 { 0 } else { !0u32 << (32 - mask) };
                let masked = bits & mask_bits;
                let canonical_ip = std::net::Ipv4Addr::from(masked);
                return format!("{}/{}", canonical_ip, mask);
            }
        }
        value.to_string()
    }

    /// Canonicalize an IPv6 address value.
    ///
    /// Normalizes CIDR notation by applying the mask to the address.
    pub fn canonicalize_ipv6(value: &str) -> String {
        if let Some((addr, mask_str)) = value.split_once('/')
            && let Ok(mask) = mask_str.parse::<u8>()
        {
            if mask >= 128 {
                // /128 is a single host, remove the suffix
                return addr.to_string();
            }
            if let Ok(ip) = addr.parse::<std::net::Ipv6Addr>() {
                let bits = u128::from(ip);
                let mask_bits = if mask == 0 { 0 } else { !0u128 << (128 - mask) };
                let masked = bits & mask_bits;
                let canonical_ip = std::net::Ipv6Addr::from(masked);
                return format!("{}/{}", canonical_ip, mask);
            }
        }
        value.to_string()
    }

    /// Canonicalize a Windows registry key value.
    ///
    /// Lowercases the key for case-insensitive comparison.
    pub fn canonicalize_windows_registry_key(value: &str) -> String {
        value.to_lowercase()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_canonicalize_ipv4() {
            assert_eq!(canonicalize_ipv4("192.168.1.100/24"), "192.168.1.0/24");
            assert_eq!(canonicalize_ipv4("10.0.0.1/32"), "10.0.0.1");
            assert_eq!(canonicalize_ipv4("192.168.1.1"), "192.168.1.1");
        }

        #[test]
        fn test_canonicalize_ipv6() {
            assert_eq!(canonicalize_ipv6("2001:db8::1/128"), "2001:db8::1");
            assert_eq!(canonicalize_ipv6("2001:db8::1"), "2001:db8::1");
        }

        #[test]
        fn test_canonicalize_windows_registry_key() {
            assert_eq!(
                canonicalize_windows_registry_key("HKEY_LOCAL_MACHINE\\SOFTWARE"),
                "hkey_local_machine\\software"
            );
        }
    }
}

// ============================================================================
// DNF Transformation
// ============================================================================

impl PatternStructure {
    /// Unwrap single-element vec or wrap in constructor.
    fn simplify_single<F>(operands: Vec<Self>, constructor: F) -> Self
    where
        F: FnOnce(Vec<Self>) -> Self,
    {
        match &operands[..] {
            [single] => single.clone(),
            _ => constructor(operands),
        }
    }

    /// Flatten nested AND/OR structures.
    ///
    /// E.g., AND(A, AND(B, C)) -> AND(A, B, C)
    fn flatten(self) -> Self {
        match self {
            PatternStructure::And(children) => {
                let mut flattened = Vec::new();
                for child in children {
                    match child.flatten() {
                        PatternStructure::And(nested) => flattened.extend(nested),
                        other => flattened.push(other),
                    }
                }
                Self::simplify_single(flattened, PatternStructure::And)
            }
            PatternStructure::Or(children) => {
                let mut flattened = Vec::new();
                for child in children {
                    match child.flatten() {
                        PatternStructure::Or(nested) => flattened.extend(nested),
                        other => flattened.push(other),
                    }
                }
                Self::simplify_single(flattened, PatternStructure::Or)
            }
            PatternStructure::FollowedBy(children) => {
                PatternStructure::FollowedBy(children.into_iter().map(|c| c.flatten()).collect())
            }
            PatternStructure::Qualified(inner, q) => {
                PatternStructure::Qualified(Box::new(inner.flatten()), q)
            }
            other => other,
        }
    }

    /// Convert to Disjunctive Normal Form (DNF).
    ///
    /// DNF is OR of ANDs: (A AND B) OR (C AND D)
    /// This distributes AND over OR: A AND (B OR C) -> (A AND B) OR (A AND C)
    fn into_dnf(self) -> Self {
        let flattened = self.flatten();

        match flattened {
            PatternStructure::And(children) => {
                // Convert children to DNF first
                let dnf_children: Vec<_> = children.into_iter().map(|c| c.into_dnf()).collect();

                // Check if any child is an OR - if so, distribute
                let or_idx = dnf_children
                    .iter()
                    .position(|c| matches!(c, PatternStructure::Or(_)));

                if let Some(idx) = or_idx {
                    let or_child = dnf_children[idx].clone();
                    let other_children: Vec<_> = dnf_children
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != idx)
                        .map(|(_, c)| c.clone())
                        .collect();

                    if let PatternStructure::Or(or_terms) = or_child {
                        // Distribute: (others AND (A OR B)) -> (others AND A) OR (others AND B)
                        let distributed: Vec<_> = or_terms
                            .into_iter()
                            .map(|term| {
                                let mut new_and = other_children.clone();
                                new_and.push(term);
                                PatternStructure::And(new_and).into_dnf()
                            })
                            .collect();
                        PatternStructure::Or(distributed).flatten()
                    } else {
                        PatternStructure::And(dnf_children)
                    }
                } else {
                    PatternStructure::And(dnf_children)
                }
            }
            PatternStructure::Or(children) => {
                PatternStructure::Or(children.into_iter().map(|c| c.into_dnf()).collect()).flatten()
            }
            other => other,
        }
    }

    /// Apply absorption rules.
    ///
    /// A AND (A OR B) -> A
    /// A OR (A AND B) -> A
    fn absorb(self) -> Self {
        match self {
            PatternStructure::And(children) => {
                // First recursively absorb children
                let absorbed: Vec<_> = children.into_iter().map(|c| c.absorb()).collect();
                // Then apply absorption at this level
                Self::apply_absorption(absorbed, false)
            }
            PatternStructure::Or(children) => {
                // First recursively absorb children
                let absorbed: Vec<_> = children.into_iter().map(|c| c.absorb()).collect();
                // Then apply absorption at this level
                Self::apply_absorption(absorbed, true)
            }
            other => other,
        }
    }

    /// Apply absorption rules to a list of operands.
    ///
    /// For AND: if A is found and (A OR B) is also found, remove (A OR B)
    /// For OR: if A is found and (A AND B) is also found, remove (A AND B)
    ///
    /// `is_or` indicates whether we're processing an OR expression (true) or AND (false).
    fn apply_absorption(operands: Vec<PatternStructure>, is_or: bool) -> Self {
        let constructor = if is_or {
            PatternStructure::Or
        } else {
            PatternStructure::And
        };

        if operands.is_empty() {
            return constructor(vec![]);
        }

        if operands.len() == 1 {
            return Self::simplify_single(operands, constructor);
        }

        let mut to_delete: std::collections::HashSet<usize> = std::collections::HashSet::new();

        // Check each pair (i, j) - if child_i is contained in child_j, delete child_j
        for i in 0..operands.len() {
            if to_delete.contains(&i) {
                continue;
            }

            for j in 0..operands.len() {
                if i == j || to_delete.contains(&j) {
                    continue;
                }

                let child_i = &operands[i];
                let child_j = &operands[j];

                // child_j must be a compound expression with the "secondary" operator
                // For AND parent: secondary is OR
                // For OR parent: secondary is AND
                let is_secondary = if is_or {
                    matches!(child_j, PatternStructure::And(_))
                } else {
                    matches!(child_j, PatternStructure::Or(_))
                };

                if !is_secondary {
                    continue;
                }

                // Get child_j's operands
                let child_j_operands = match child_j {
                    PatternStructure::And(ops) => ops,
                    PatternStructure::Or(ops) => ops,
                    _ => continue,
                };

                // Simple check: is child_i directly contained in child_j's operands?
                if child_j_operands.iter().any(|op| op == child_i) {
                    to_delete.insert(j);
                    continue;
                }

                // More complex check: if child_i has the same operator as child_j,
                // check if ALL of child_i's operands are contained in child_j
                let child_i_same_op = if is_or {
                    matches!(child_i, PatternStructure::And(_))
                } else {
                    matches!(child_i, PatternStructure::Or(_))
                };

                if child_i_same_op {
                    let child_i_operands = match child_i {
                        PatternStructure::And(ops) => ops,
                        PatternStructure::Or(ops) => ops,
                        _ => continue,
                    };

                    // Check if all operands of child_i are in child_j
                    let all_contained = child_i_operands
                        .iter()
                        .all(|op| child_j_operands.iter().any(|jop| jop == op));

                    if all_contained {
                        to_delete.insert(j);
                    }
                }
            }
        }

        // Build result without deleted indices
        let result: Vec<_> = operands
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| !to_delete.contains(idx))
            .map(|(_, op)| op)
            .collect();

        Self::simplify_single(result, constructor)
    }

    /// Apply transformations until no changes (settling).
    fn settle(self) -> Self {
        let mut current = self;
        loop {
            let flattened = current.clone().flatten();
            let absorbed = flattened.absorb();
            if absorbed == current {
                return current;
            }
            current = absorbed;
        }
    }

    /// Full normalization: flatten, convert to DNF, then settle.
    fn normalize(self) -> Self {
        self.flatten().into_dnf().settle()
    }
}

/// Check if two STIX patterns are semantically equivalent.
///
/// # Example
///
/// ```rust,ignore
/// use stix2::pattern_equivalence::equivalent_patterns;
///
/// let result = equivalent_patterns(
///     "[file:name = 'test.exe']",
///     "[file:name='test.exe']"
/// );
/// assert!(result.unwrap());
/// ```
pub fn equivalent_patterns(pattern1: &str, pattern2: &str) -> Result<bool> {
    let ast1 = parse_pattern(pattern1)?;
    let ast2 = parse_pattern(pattern2)?;

    let norm1 = normalize_expression(&ast1);
    let norm2 = normalize_expression(&ast2);

    Ok(compare_patterns(&norm1, &norm2) == Ordering::Equal)
}

/// Find patterns from a collection that are equivalent to a search pattern.
///
/// This is more efficient than calling `equivalent_patterns` in a loop
/// because the search pattern is only normalized once.
pub fn find_equivalent_patterns<'a, I>(search_pattern: &str, patterns: I) -> Result<Vec<String>>
where
    I: IntoIterator<Item = &'a str>,
{
    let search_ast = parse_pattern(search_pattern)?;
    let norm_search = normalize_expression(&search_ast);

    let mut results = Vec::new();

    for pattern in patterns {
        if let Ok(ast) = parse_pattern(pattern) {
            let norm = normalize_expression(&ast);
            if compare_patterns(&norm_search, &norm) == Ordering::Equal {
                results.push(pattern.to_string());
            }
        }
    }

    Ok(results)
}

/// Calculate similarity score between two patterns (0-100).
pub fn pattern_similarity(pattern1: &str, pattern2: &str) -> Result<f64> {
    let ast1 = parse_pattern(pattern1)?;
    let ast2 = parse_pattern(pattern2)?;

    let norm1 = normalize_expression(&ast1);
    let norm2 = normalize_expression(&ast2);

    Ok(calculate_pattern_similarity(&norm1, &norm2))
}

/// Normalized pattern representation for comparison.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NormalizedPattern {
    /// Comparisons in canonical order
    comparisons: Vec<NormalizedComparison>,
    /// Structure representation
    structure: PatternStructure,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PatternStructure {
    Single,
    And(Vec<PatternStructure>),
    Or(Vec<PatternStructure>),
    FollowedBy(Vec<PatternStructure>),
    Qualified(Box<PatternStructure>, String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NormalizedComparison {
    /// Object type (e.g., "file", "network-traffic")
    object_type: String,
    /// Property path (e.g., "name", "hashes.SHA-256")
    property_path: String,
    /// Operator
    operator: NormalizedOperator,
    /// Value (normalized)
    value: NormalizedValue,
    /// Whether negated
    negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NormalizedOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Matches,
    Like,
    In,
    IsSubset,
    IsSuperset,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NormalizedValue {
    String(String),
    Integer(i64),
    Float(OrderedFloat),
    Boolean(bool),
    Timestamp(String),
    Binary(Vec<u8>),
    Hex(String),
    List(Vec<NormalizedValue>),
}

/// Wrapper for f64 that implements Ord (for sorting purposes)
#[derive(Debug, Clone, PartialEq)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

/// Normalize a pattern expression for comparison.
fn normalize_expression(expr: &PatternExpression) -> NormalizedPattern {
    let mut comparisons = Vec::new();
    let structure = collect_pattern_info(expr, &mut comparisons);

    // Apply DNF normalization to structure
    let normalized_structure = structure.normalize();

    // Sort comparisons for canonical ordering
    comparisons.sort();

    // Deduplicate
    comparisons.dedup();

    NormalizedPattern {
        comparisons,
        structure: normalized_structure,
    }
}

fn collect_pattern_info(
    expr: &PatternExpression,
    comparisons: &mut Vec<NormalizedComparison>,
) -> PatternStructure {
    match expr {
        PatternExpression::Comparison(comp) => {
            comparisons.push(normalize_comparison(comp));
            PatternStructure::Single
        }
        PatternExpression::And(left, right) => {
            let left_struct = collect_pattern_info(left, comparisons);
            let right_struct = collect_pattern_info(right, comparisons);
            PatternStructure::And(vec![left_struct, right_struct])
        }
        PatternExpression::Or(left, right) => {
            let left_struct = collect_pattern_info(left, comparisons);
            let right_struct = collect_pattern_info(right, comparisons);
            PatternStructure::Or(vec![left_struct, right_struct])
        }
        PatternExpression::FollowedBy(left, right) => {
            let left_struct = collect_pattern_info(left, comparisons);
            let right_struct = collect_pattern_info(right, comparisons);
            PatternStructure::FollowedBy(vec![left_struct, right_struct])
        }
        PatternExpression::Qualified(inner, qualifier) => {
            let inner_struct = collect_pattern_info(inner, comparisons);
            PatternStructure::Qualified(Box::new(inner_struct), format!("{}", qualifier))
        }
    }
}

fn normalize_comparison(comp: &ComparisonExpression) -> NormalizedComparison {
    // Apply special value canonicalization based on object type
    let canonical_value = canonicalize_value(&comp.object_type, &comp.object_path, &comp.value);

    NormalizedComparison {
        object_type: comp.object_type.clone(),
        property_path: comp.object_path.clone(),
        operator: normalize_operator(&comp.operator),
        value: canonical_value,
        negated: comp.negated,
    }
}

/// Apply special value canonicalization based on object type and property.
fn canonicalize_value(
    object_type: &str,
    property_path: &str,
    value: &PatternValue,
) -> NormalizedValue {
    match (object_type, property_path, value) {
        // IPv4 address CIDR normalization
        ("ipv4-addr", "value", PatternValue::String(s)) => {
            NormalizedValue::String(specials::canonicalize_ipv4(s))
        }
        // IPv6 address CIDR normalization
        ("ipv6-addr", "value", PatternValue::String(s)) => {
            NormalizedValue::String(specials::canonicalize_ipv6(s))
        }
        // Windows registry key case-insensitive normalization
        ("windows-registry-key", "key", PatternValue::String(s)) => {
            NormalizedValue::String(specials::canonicalize_windows_registry_key(s))
        }
        // Default normalization
        _ => normalize_value(value),
    }
}

fn normalize_operator(op: &ComparisonOperator) -> NormalizedOperator {
    match op {
        ComparisonOperator::Equal => NormalizedOperator::Equal,
        ComparisonOperator::NotEqual => NormalizedOperator::NotEqual,
        ComparisonOperator::LessThan => NormalizedOperator::LessThan,
        ComparisonOperator::LessThanOrEqual => NormalizedOperator::LessThanEqual,
        ComparisonOperator::GreaterThan => NormalizedOperator::GreaterThan,
        ComparisonOperator::GreaterThanOrEqual => NormalizedOperator::GreaterThanEqual,
        ComparisonOperator::Matches => NormalizedOperator::Matches,
        ComparisonOperator::Like => NormalizedOperator::Like,
        ComparisonOperator::In => NormalizedOperator::In,
        ComparisonOperator::IsSubset => NormalizedOperator::IsSubset,
        ComparisonOperator::IsSuperset => NormalizedOperator::IsSuperset,
    }
}

fn normalize_value(val: &PatternValue) -> NormalizedValue {
    match val {
        PatternValue::String(s) => NormalizedValue::String(s.clone()),
        PatternValue::Integer(i) => NormalizedValue::Integer(*i),
        PatternValue::Float(f) => NormalizedValue::Float(OrderedFloat(*f)),
        PatternValue::Boolean(b) => NormalizedValue::Boolean(*b),
        PatternValue::Timestamp(t) => NormalizedValue::Timestamp(t.clone()),
        PatternValue::Binary(b) => NormalizedValue::Binary(b.clone()),
        PatternValue::Hex(h) => NormalizedValue::Hex(h.to_lowercase()),
        PatternValue::List(items) => {
            let mut normalized: Vec<_> = items.iter().map(normalize_value).collect();
            normalized.sort();
            NormalizedValue::List(normalized)
        }
    }
}

/// Compare two normalized patterns.
fn compare_patterns(p1: &NormalizedPattern, p2: &NormalizedPattern) -> Ordering {
    // First compare comparisons
    let comp_cmp = p1.comparisons.cmp(&p2.comparisons);
    if comp_cmp != Ordering::Equal {
        return comp_cmp;
    }

    // Then compare structure
    p1.structure.cmp(&p2.structure)
}

/// Calculate similarity between two normalized patterns.
fn calculate_pattern_similarity(p1: &NormalizedPattern, p2: &NormalizedPattern) -> f64 {
    if p1 == p2 {
        return 100.0;
    }

    // Count matching comparisons
    let set1: BTreeSet<_> = p1.comparisons.iter().collect();
    let set2: BTreeSet<_> = p2.comparisons.iter().collect();

    let intersection = set1.intersection(&set2).count();
    let union = set1.union(&set2).count();

    if union == 0 {
        return 100.0;
    }

    // Base similarity on comparison overlap
    let comparison_similarity = (intersection as f64 / union as f64) * 100.0;

    // Adjust for structural similarity
    let structure_match = p1.structure == p2.structure;
    if structure_match {
        comparison_similarity
    } else {
        comparison_similarity * 0.8 // Penalize structural differences
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivalent_identical_patterns() {
        let result = equivalent_patterns("[file:name = 'test.exe']", "[file:name = 'test.exe']");
        assert!(result.unwrap());
    }

    #[test]
    fn test_equivalent_whitespace_difference() {
        let result = equivalent_patterns("[file:name = 'test.exe']", "[file:name='test.exe']");
        assert!(result.unwrap());
    }

    #[test]
    fn test_not_equivalent_different_values() {
        let result = equivalent_patterns("[file:name = 'test.exe']", "[file:name = 'other.exe']");
        assert!(!result.unwrap());
    }

    #[test]
    fn test_not_equivalent_different_operators() {
        let result = equivalent_patterns("[file:size = 100]", "[file:size > 100]");
        assert!(!result.unwrap());
    }

    #[test]
    fn test_pattern_similarity_identical() {
        let result = pattern_similarity("[file:name = 'test.exe']", "[file:name = 'test.exe']");
        assert_eq!(result.unwrap(), 100.0);
    }

    #[test]
    fn test_pattern_similarity_different() {
        let result = pattern_similarity("[file:name = 'test.exe']", "[file:name = 'other.exe']");
        assert!(result.unwrap() < 100.0);
    }

    #[test]
    fn test_find_equivalent_patterns() {
        let patterns = vec![
            "[file:name = 'test.exe']",
            "[file:name = 'other.exe']",
            "[file:name='test.exe']",
            "[process:name = 'cmd.exe']",
        ];

        let result = find_equivalent_patterns("[file:name = 'test.exe']", patterns).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_dnf_flatten() {
        // Test that nested structures get flattened
        let nested = PatternStructure::And(vec![
            PatternStructure::Single,
            PatternStructure::And(vec![PatternStructure::Single, PatternStructure::Single]),
        ]);

        let flattened = nested.flatten();
        if let PatternStructure::And(children) = flattened {
            assert_eq!(children.len(), 3);
        } else {
            panic!("Expected And structure");
        }
    }

    #[test]
    fn test_special_value_ipv4_cidr() {
        // Test that different representations of same CIDR are equivalent
        let value1 = specials::canonicalize_ipv4("192.168.1.100/24");
        let value2 = specials::canonicalize_ipv4("192.168.1.0/24");
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_special_value_windows_registry() {
        // Test case-insensitive registry key comparison
        let value1 = specials::canonicalize_windows_registry_key("HKEY_LOCAL_MACHINE\\SOFTWARE");
        let value2 = specials::canonicalize_windows_registry_key("hkey_local_machine\\software");
        assert_eq!(value1, value2);
    }

    // =========================================================================
    // Absorption Tests
    // =========================================================================

    #[test]
    fn test_absorption_and_simple() {
        // A AND (A OR B) -> A
        // Structure: And([Single, Or([Single, Single])])
        // Where the first Single == first element of Or
        let a = PatternStructure::Single;
        let b = PatternStructure::Qualified(Box::new(PatternStructure::Single), "Q".to_string());
        let a_or_b = PatternStructure::Or(vec![a.clone(), b]);
        let expr = PatternStructure::And(vec![a, a_or_b]);

        let absorbed = expr.absorb();

        // Should absorb to just A (Single)
        assert_eq!(absorbed, PatternStructure::Single);
    }

    #[test]
    fn test_absorption_or_simple() {
        // A OR (A AND B) -> A
        let a = PatternStructure::Single;
        let b = PatternStructure::Qualified(Box::new(PatternStructure::Single), "Q".to_string());
        let a_and_b = PatternStructure::And(vec![a.clone(), b]);
        let expr = PatternStructure::Or(vec![a, a_and_b]);

        let absorbed = expr.absorb();

        // Should absorb to just A (Single)
        assert_eq!(absorbed, PatternStructure::Single);
    }

    #[test]
    fn test_absorption_and_multiple() {
        // A AND B AND (A OR C) -> A AND B
        // Because A is contained in (A OR C)
        let a = PatternStructure::Single;
        let b = PatternStructure::Qualified(Box::new(PatternStructure::Single), "B".to_string());
        let c = PatternStructure::Qualified(Box::new(PatternStructure::Single), "C".to_string());
        let a_or_c = PatternStructure::Or(vec![a.clone(), c]);
        let expr = PatternStructure::And(vec![a.clone(), b.clone(), a_or_c]);

        let absorbed = expr.absorb();

        // Should absorb to And([A, B])
        if let PatternStructure::And(children) = absorbed {
            assert_eq!(children.len(), 2);
            assert!(children.contains(&a));
            assert!(children.contains(&b));
        } else {
            panic!("Expected And structure, got {:?}", absorbed);
        }
    }

    #[test]
    fn test_absorption_or_multiple() {
        // A OR B OR (A AND C) -> A OR B
        // Because A is contained in (A AND C)
        let a = PatternStructure::Single;
        let b = PatternStructure::Qualified(Box::new(PatternStructure::Single), "B".to_string());
        let c = PatternStructure::Qualified(Box::new(PatternStructure::Single), "C".to_string());
        let a_and_c = PatternStructure::And(vec![a.clone(), c]);
        let expr = PatternStructure::Or(vec![a.clone(), b.clone(), a_and_c]);

        let absorbed = expr.absorb();

        // Should absorb to Or([A, B])
        if let PatternStructure::Or(children) = absorbed {
            assert_eq!(children.len(), 2);
            assert!(children.contains(&a));
            assert!(children.contains(&b));
        } else {
            panic!("Expected Or structure, got {:?}", absorbed);
        }
    }

    #[test]
    fn test_absorption_no_change() {
        // A AND B - nothing to absorb
        let a = PatternStructure::Single;
        let b = PatternStructure::Qualified(Box::new(PatternStructure::Single), "B".to_string());
        let expr = PatternStructure::And(vec![a, b]);

        let absorbed = expr.absorb();

        // Should remain unchanged
        if let PatternStructure::And(children) = absorbed {
            assert_eq!(children.len(), 2);
        } else {
            panic!("Expected And structure");
        }
    }

    #[test]
    fn test_absorption_nested() {
        // Test that absorption works recursively
        // (A OR (A AND B)) AND C -> A AND C
        let a = PatternStructure::Single;
        let b = PatternStructure::Qualified(Box::new(PatternStructure::Single), "B".to_string());
        let c = PatternStructure::Qualified(Box::new(PatternStructure::Single), "C".to_string());

        let a_and_b = PatternStructure::And(vec![a.clone(), b]);
        let inner_or = PatternStructure::Or(vec![a.clone(), a_and_b]); // This should absorb to A
        let expr = PatternStructure::And(vec![inner_or, c.clone()]);

        let absorbed = expr.absorb();

        // Inner OR absorbs to A, result is And([A, C])
        if let PatternStructure::And(children) = absorbed {
            assert_eq!(children.len(), 2);
            assert!(children.contains(&a));
            assert!(children.contains(&c));
        } else {
            panic!("Expected And structure, got {:?}", absorbed);
        }
    }

    #[test]
    fn test_absorption_flattened_containment() {
        // Test the "flattened containment" case
        // (A AND B) OR (A AND B AND C) -> (A AND B)
        // Because all operands of (A AND B) are in (A AND B AND C)
        let a = PatternStructure::Single;
        let b = PatternStructure::Qualified(Box::new(PatternStructure::Single), "B".to_string());
        let c = PatternStructure::Qualified(Box::new(PatternStructure::Single), "C".to_string());

        let a_and_b = PatternStructure::And(vec![a.clone(), b.clone()]);
        let a_and_b_and_c = PatternStructure::And(vec![a.clone(), b.clone(), c]);
        let expr = PatternStructure::Or(vec![a_and_b, a_and_b_and_c]);

        let absorbed = expr.absorb();

        // Should absorb to just (A AND B)
        if let PatternStructure::And(children) = absorbed {
            assert_eq!(children.len(), 2);
            assert!(children.contains(&a));
            assert!(children.contains(&b));
        } else {
            panic!("Expected And structure, got {:?}", absorbed);
        }
    }
}
