//! STIX Object Graph Analysis
//!
//! This module provides utilities for analyzing and comparing STIX object graphs,
//! including graph equivalence checking and traversal.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::core::stix_object::StixObject;
use crate::equivalence::{DEFAULT_THRESHOLD, object_similarity};
use crate::relationship::Relationship;

/// Result of matching objects between two graphs.
/// Contains: (matched pairs with similarity, unmatched from first, unmatched from second)
type MatchResult = (Vec<(String, String, f64)>, Vec<String>, Vec<String>);

/// A graph of STIX objects and their relationships.
#[derive(Debug, Clone, Default)]
pub struct StixGraph {
    /// All objects in the graph, indexed by ID.
    objects: HashMap<String, StixObject>,
    /// Adjacency list: source_id -> [(target_id, relationship_type)]
    edges: HashMap<String, Vec<(String, String)>>,
    /// Reverse adjacency list: target_id -> [(source_id, relationship_type)]
    reverse_edges: HashMap<String, Vec<(String, String)>>,
}

impl StixGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a graph from a collection of STIX objects.
    pub fn from_objects(objects: Vec<StixObject>) -> Self {
        let mut graph = Self::new();
        for obj in objects {
            graph.add_object(obj);
        }
        graph
    }

    /// Add an object to the graph.
    pub fn add_object(&mut self, object: StixObject) {
        let id = object.id().to_string();

        // If it's a relationship, add edges
        if let StixObject::Relationship(ref rel) = object {
            let source = rel.source_ref.to_string();
            let target = rel.target_ref.to_string();
            let rel_type = rel.relationship_type.clone();

            self.edges
                .entry(source.clone())
                .or_default()
                .push((target.clone(), rel_type.clone()));

            self.reverse_edges
                .entry(target)
                .or_default()
                .push((source, rel_type));
        }

        self.objects.insert(id, object);
    }

    /// Get an object by ID.
    pub fn get(&self, id: &str) -> Option<&StixObject> {
        self.objects.get(id)
    }

    /// Get all objects in the graph.
    pub fn objects(&self) -> impl Iterator<Item = &StixObject> {
        self.objects.values()
    }

    /// Get the number of objects in the graph.
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Get outgoing relationships from an object.
    pub fn outgoing(&self, id: &str) -> Vec<(&StixObject, &str)> {
        self.edges
            .get(id)
            .map(|edges| {
                edges
                    .iter()
                    .filter_map(|(target, rel_type)| {
                        self.objects.get(target).map(|obj| (obj, rel_type.as_str()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get incoming relationships to an object.
    pub fn incoming(&self, id: &str) -> Vec<(&StixObject, &str)> {
        self.reverse_edges
            .get(id)
            .map(|edges| {
                edges
                    .iter()
                    .filter_map(|(source, rel_type)| {
                        self.objects.get(source).map(|obj| (obj, rel_type.as_str()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all neighbors (both incoming and outgoing) of an object.
    pub fn neighbors(&self, id: &str) -> Vec<(&StixObject, &str)> {
        let mut result = self.outgoing(id);
        result.extend(self.incoming(id));
        result
    }

    /// Find all objects of a given type.
    pub fn by_type(&self, type_name: &str) -> Vec<&StixObject> {
        self.objects
            .values()
            .filter(|obj| obj.type_name() == type_name)
            .collect()
    }

    /// Perform a breadth-first traversal from a starting object.
    pub fn bfs(&self, start_id: &str, max_depth: Option<usize>) -> Vec<&StixObject> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back((start_id.to_string(), 0usize));
        visited.insert(start_id.to_string());

        while let Some((current_id, depth)) = queue.pop_front() {
            if let Some(max) = max_depth
                && depth > max
            {
                continue;
            }

            if let Some(obj) = self.objects.get(&current_id) {
                result.push(obj);

                // Add neighbors to queue
                for (target, _) in self.edges.get(&current_id).unwrap_or(&vec![]) {
                    if !visited.contains(target) {
                        visited.insert(target.clone());
                        queue.push_back((target.clone(), depth + 1));
                    }
                }
            }
        }

        result
    }

    /// Get the subgraph containing only the specified object types.
    pub fn filter_by_types(&self, types: &[&str]) -> StixGraph {
        let type_set: HashSet<&str> = types.iter().copied().collect();
        let filtered_objects: Vec<StixObject> = self
            .objects
            .values()
            .filter(|obj| type_set.contains(obj.type_name()))
            .cloned()
            .collect();

        StixGraph::from_objects(filtered_objects)
    }

    /// Get all relationship objects in the graph.
    pub fn relationships(&self) -> Vec<&Relationship> {
        self.objects
            .values()
            .filter_map(|obj| {
                if let StixObject::Relationship(rel) = obj {
                    Some(rel)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find paths between two objects.
    pub fn find_paths(&self, start_id: &str, end_id: &str, max_length: usize) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut current_path = vec![start_id.to_string()];
        self.find_paths_recursive(start_id, end_id, max_length, &mut current_path, &mut paths);
        paths
    }

    fn find_paths_recursive(
        &self,
        current: &str,
        target: &str,
        remaining: usize,
        current_path: &mut Vec<String>,
        all_paths: &mut Vec<Vec<String>>,
    ) {
        if current == target {
            all_paths.push(current_path.clone());
            return;
        }

        if remaining == 0 {
            return;
        }

        if let Some(edges) = self.edges.get(current) {
            for (next, _) in edges {
                if !current_path.contains(next) {
                    current_path.push(next.clone());
                    self.find_paths_recursive(next, target, remaining - 1, current_path, all_paths);
                    current_path.pop();
                }
            }
        }
    }

    /// Calculate graph statistics.
    pub fn statistics(&self) -> GraphStatistics {
        let mut type_counts: HashMap<String, usize> = HashMap::new();

        for obj in self.objects.values() {
            *type_counts.entry(obj.type_name().to_string()).or_insert(0) += 1;
        }

        let relationship_count = type_counts.get("relationship").copied().unwrap_or(0);
        let sighting_count = type_counts.get("sighting").copied().unwrap_or(0);
        let sdo_count = self
            .objects
            .values()
            .filter(|obj| obj.is_domain_object())
            .count();
        let sco_count = self
            .objects
            .values()
            .filter(|obj| obj.is_cyber_observable())
            .count();

        GraphStatistics {
            total_objects: self.objects.len(),
            sdo_count,
            sro_count: relationship_count + sighting_count,
            sco_count,
            relationship_count,
            sighting_count,
            type_counts,
        }
    }
}

/// Statistics about a STIX graph.
#[derive(Debug, Clone)]
pub struct GraphStatistics {
    /// Total number of objects.
    pub total_objects: usize,
    /// Number of SDOs.
    pub sdo_count: usize,
    /// Number of SROs.
    pub sro_count: usize,
    /// Number of SCOs.
    pub sco_count: usize,
    /// Number of relationships.
    pub relationship_count: usize,
    /// Number of sightings.
    pub sighting_count: usize,
    /// Count by object type.
    pub type_counts: HashMap<String, usize>,
}

/// Result of comparing two STIX graphs.
#[derive(Debug, Clone)]
pub struct GraphEquivalenceResult {
    /// Overall similarity score (0-100).
    pub similarity: f64,
    /// Whether the graphs are considered equivalent.
    pub equivalent: bool,
    /// Matched object pairs (graph1_id, graph2_id, similarity).
    pub matched_objects: Vec<(String, String, f64)>,
    /// Unmatched objects from graph 1.
    pub unmatched_graph1: Vec<String>,
    /// Unmatched objects from graph 2.
    pub unmatched_graph2: Vec<String>,
    /// Structural similarity (based on relationship patterns).
    pub structural_similarity: f64,
    /// Content similarity (based on object content).
    pub content_similarity: f64,
}

/// Options for graph equivalence comparison.
#[derive(Debug, Clone)]
pub struct GraphEquivalenceOptions {
    /// Threshold for considering objects equivalent (0-100).
    pub object_threshold: f64,
    /// Threshold for considering graphs equivalent (0-100).
    pub graph_threshold: f64,
    /// Weight for content similarity (0-1).
    pub content_weight: f64,
    /// Weight for structural similarity (0-1).
    pub structure_weight: f64,
    /// Whether to ignore relationship objects in comparison.
    pub ignore_relationships: bool,
    /// Object types to include (empty = all).
    pub include_types: Vec<String>,
    /// Object types to exclude.
    pub exclude_types: Vec<String>,
}

impl Default for GraphEquivalenceOptions {
    fn default() -> Self {
        Self {
            object_threshold: DEFAULT_THRESHOLD,
            graph_threshold: 70.0,
            content_weight: 0.7,
            structure_weight: 0.3,
            ignore_relationships: false,
            include_types: vec![],
            exclude_types: vec![],
        }
    }
}

/// Compare two STIX graphs for equivalence.
pub fn graph_equivalence(
    graph1: &StixGraph,
    graph2: &StixGraph,
    options: Option<GraphEquivalenceOptions>,
) -> GraphEquivalenceResult {
    let opts = options.unwrap_or_default();

    // Get objects to compare (filtering as specified)
    let objects1 = filter_objects_for_comparison(graph1, &opts);
    let objects2 = filter_objects_for_comparison(graph2, &opts);

    // Build similarity matrix
    let mut similarity_matrix: Vec<Vec<f64>> = Vec::new();
    for obj1 in &objects1 {
        let mut row = Vec::new();
        for obj2 in &objects2 {
            row.push(object_similarity(obj1, obj2));
        }
        similarity_matrix.push(row);
    }

    // Find best matches using greedy algorithm
    let (matched, unmatched1, unmatched2) = find_best_matches(
        &objects1,
        &objects2,
        &similarity_matrix,
        opts.object_threshold,
    );

    // Calculate content similarity
    let content_similarity = if matched.is_empty() {
        0.0
    } else {
        matched.iter().map(|(_, _, s)| s).sum::<f64>() / matched.len() as f64
    };

    // Calculate structural similarity
    let structural_similarity = calculate_structural_similarity(graph1, graph2, &matched, &opts);

    // Calculate overall similarity
    let overall_similarity =
        opts.content_weight * content_similarity + opts.structure_weight * structural_similarity;

    // Adjust for unmatched objects
    let total_objects = objects1.len() + objects2.len();
    let matched_count = matched.len() * 2;
    let match_ratio = if total_objects > 0 {
        matched_count as f64 / total_objects as f64
    } else {
        1.0
    };

    let adjusted_similarity = overall_similarity * match_ratio;

    GraphEquivalenceResult {
        similarity: adjusted_similarity,
        equivalent: adjusted_similarity >= opts.graph_threshold,
        matched_objects: matched,
        unmatched_graph1: unmatched1,
        unmatched_graph2: unmatched2,
        structural_similarity,
        content_similarity,
    }
}

fn filter_objects_for_comparison<'a>(
    graph: &'a StixGraph,
    opts: &GraphEquivalenceOptions,
) -> Vec<&'a StixObject> {
    graph
        .objects()
        .filter(|obj| {
            // Filter by type
            if opts.ignore_relationships && obj.type_name() == "relationship" {
                return false;
            }

            if !opts.include_types.is_empty()
                && !opts.include_types.contains(&obj.type_name().to_string())
            {
                return false;
            }

            if opts.exclude_types.contains(&obj.type_name().to_string()) {
                return false;
            }

            true
        })
        .collect()
}

fn find_best_matches<'a>(
    objects1: &[&'a StixObject],
    objects2: &[&'a StixObject],
    similarity_matrix: &[Vec<f64>],
    threshold: f64,
) -> MatchResult {
    let mut matched = Vec::new();
    let mut used1: HashSet<usize> = HashSet::new();
    let mut used2: HashSet<usize> = HashSet::new();

    // Build list of all similarities above threshold
    let mut candidates: Vec<(usize, usize, f64)> = Vec::new();
    for (i, row) in similarity_matrix.iter().enumerate() {
        for (j, &sim) in row.iter().enumerate() {
            if sim >= threshold {
                candidates.push((i, j, sim));
            }
        }
    }

    // Sort by similarity descending
    candidates.sort_by(|a, b| b.2.total_cmp(&a.2));

    // Greedy matching
    for (i, j, sim) in candidates {
        if !used1.contains(&i) && !used2.contains(&j) {
            matched.push((
                objects1[i].id().to_string(),
                objects2[j].id().to_string(),
                sim,
            ));
            used1.insert(i);
            used2.insert(j);
        }
    }

    // Find unmatched
    let unmatched1: Vec<String> = objects1
        .iter()
        .enumerate()
        .filter(|(i, _)| !used1.contains(i))
        .map(|(_, obj)| obj.id().to_string())
        .collect();

    let unmatched2: Vec<String> = objects2
        .iter()
        .enumerate()
        .filter(|(i, _)| !used2.contains(i))
        .map(|(_, obj)| obj.id().to_string())
        .collect();

    (matched, unmatched1, unmatched2)
}

fn calculate_structural_similarity(
    graph1: &StixGraph,
    graph2: &StixGraph,
    matched: &[(String, String, f64)],
    _opts: &GraphEquivalenceOptions,
) -> f64 {
    if matched.is_empty() {
        return 0.0;
    }

    // Create mapping from graph1 IDs to graph2 IDs
    let id_map: HashMap<&str, &str> = matched
        .iter()
        .map(|(id1, id2, _)| (id1.as_str(), id2.as_str()))
        .collect();

    let mut edge_matches = 0;
    let mut total_edges = 0;

    // Check if edges in graph1 have corresponding edges in graph2
    for rel in graph1.relationships() {
        total_edges += 1;

        let source1 = rel.source_ref.to_string();
        let target1 = rel.target_ref.to_string();

        if let (Some(&source2), Some(&target2)) =
            (id_map.get(source1.as_str()), id_map.get(target1.as_str()))
        {
            // Check if corresponding edge exists in graph2
            if let Some(edges) = graph2.edges.get(source2) {
                for (t, rt) in edges {
                    if t == target2 && rt == &rel.relationship_type {
                        edge_matches += 1;
                        break;
                    }
                }
            }
        }
    }

    // Also count edges in graph2 for symmetric comparison
    let graph2_edges = graph2.relationships().len();
    let total_edges = total_edges + graph2_edges;

    if total_edges == 0 {
        return 100.0; // Both graphs have no relationships
    }

    (edge_matches * 2) as f64 / total_edges as f64 * 100.0
}

/// Check if two graphs are semantically equivalent.
pub fn graphs_equivalent(graph1: &StixGraph, graph2: &StixGraph, threshold: Option<f64>) -> bool {
    let opts = GraphEquivalenceOptions {
        graph_threshold: threshold.unwrap_or(DEFAULT_THRESHOLD),
        ..Default::default()
    };
    graph_equivalence(graph1, graph2, Some(opts)).equivalent
}

/// Calculate the similarity between two graphs.
pub fn graph_similarity(graph1: &StixGraph, graph2: &StixGraph) -> f64 {
    graph_equivalence(graph1, graph2, None).similarity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::{Indicator, Malware};
    use crate::relationship::Relationship;
    use crate::vocab::{MalwareType, PatternType};

    fn create_test_indicator(name: &str) -> StixObject {
        StixObject::Indicator(
            Indicator::builder()
                .name(name)
                .pattern("[file:name = 'malware.exe']")
                .pattern_type(PatternType::Stix)
                .valid_from_now()
                .build()
                .unwrap(),
        )
    }

    fn create_test_malware(name: &str) -> StixObject {
        StixObject::Malware(
            Malware::builder()
                .name(name)
                .is_family(false)
                .malware_type(MalwareType::Trojan)
                .build()
                .unwrap(),
        )
    }

    #[test]
    fn test_graph_creation() {
        let mut graph = StixGraph::new();
        let indicator = create_test_indicator("Test Indicator");
        graph.add_object(indicator);

        assert_eq!(graph.len(), 1);
        assert_eq!(graph.by_type("indicator").len(), 1);
    }

    #[test]
    fn test_graph_from_objects() {
        let objects = vec![
            create_test_indicator("Indicator 1"),
            create_test_indicator("Indicator 2"),
            create_test_malware("Malware 1"),
        ];

        let graph = StixGraph::from_objects(objects);
        assert_eq!(graph.len(), 3);
        assert_eq!(graph.by_type("indicator").len(), 2);
        assert_eq!(graph.by_type("malware").len(), 1);
    }

    #[test]
    fn test_graph_relationships() {
        let indicator = create_test_indicator("Test Indicator");
        let malware = create_test_malware("Test Malware");

        let ind_id = indicator.id().clone();
        let mal_id = malware.id().clone();

        let relationship = StixObject::Relationship(
            Relationship::builder()
                .source_ref(ind_id.clone())
                .target_ref(mal_id)
                .relationship_type("indicates")
                .build()
                .unwrap(),
        );

        let graph = StixGraph::from_objects(vec![indicator, malware, relationship]);

        assert_eq!(graph.len(), 3);
        assert_eq!(graph.relationships().len(), 1);

        let outgoing = graph.outgoing(&ind_id.to_string());
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].1, "indicates");
    }

    #[test]
    fn test_graph_statistics() {
        let objects = vec![
            create_test_indicator("Indicator 1"),
            create_test_indicator("Indicator 2"),
            create_test_malware("Malware 1"),
        ];

        let graph = StixGraph::from_objects(objects);
        let stats = graph.statistics();

        assert_eq!(stats.total_objects, 3);
        assert_eq!(stats.sdo_count, 3);
        assert_eq!(stats.type_counts.get("indicator"), Some(&2));
        assert_eq!(stats.type_counts.get("malware"), Some(&1));
    }

    #[test]
    fn test_graph_equivalence_identical() {
        let objects1 = vec![
            create_test_indicator("Test Indicator"),
            create_test_malware("Test Malware"),
        ];
        let objects2 = objects1.clone();

        let graph1 = StixGraph::from_objects(objects1);
        let graph2 = StixGraph::from_objects(objects2);

        let result = graph_equivalence(&graph1, &graph2, None);
        assert!(result.similarity > 90.0);
        assert!(result.equivalent);
    }

    #[test]
    fn test_graph_equivalence_similar() {
        let graph1 = StixGraph::from_objects(vec![
            create_test_indicator("APT Indicator"),
            create_test_malware("APT Malware"),
        ]);

        let graph2 = StixGraph::from_objects(vec![
            create_test_indicator("APT Indicator"),
            create_test_malware("APT Malware"),
        ]);

        let result = graph_equivalence(&graph1, &graph2, None);
        assert!(result.similarity > 50.0);
    }

    #[test]
    fn test_graph_equivalence_different() {
        let graph1 = StixGraph::from_objects(vec![create_test_indicator("Indicator A")]);

        let graph2 = StixGraph::from_objects(vec![create_test_malware("Completely Different")]);

        let result = graph_equivalence(&graph1, &graph2, None);
        assert!(result.similarity < 50.0);
        assert!(!result.equivalent);
    }

    #[test]
    fn test_bfs_traversal() {
        let indicator = create_test_indicator("Test Indicator");
        let malware = create_test_malware("Test Malware");

        let ind_id = indicator.id().clone();
        let mal_id = malware.id().clone();

        let relationship = StixObject::Relationship(
            Relationship::builder()
                .source_ref(ind_id.clone())
                .target_ref(mal_id)
                .relationship_type("indicates")
                .build()
                .unwrap(),
        );

        let graph = StixGraph::from_objects(vec![indicator, malware, relationship]);
        let traversed = graph.bfs(&ind_id.to_string(), Some(2));

        assert!(!traversed.is_empty());
    }
}
