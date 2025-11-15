// tests/v0_core_logic.rs

use llmrmgcore::*;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

fn create_test_core_with_initial_state() -> InMemoryCore {
    let mut core = InMemoryCore::new();
    let mut graph = Graph::default();
    let fixed_timestamp = DateTime::parse_from_rfc3339("2025-11-15T12:00:00Z").unwrap().with_timezone(&Utc);

    // Add a Thread, Turn, and Message
    graph.nodes.push(Node {
        id: "thread-1".to_string(),
        node_type: "Thread".to_string(),
        attrs: BTreeMap::new(),
    });
    graph.nodes.push(Node {
        id: "turn-1".to_string(),
        node_type: "Turn".to_string(),
        attrs: BTreeMap::new(),
    });
    graph.nodes.push(Node {
        id: "msg-1".to_string(),
        node_type: "Message".to_string(),
        attrs: {
            let mut map = BTreeMap::new();
            map.insert("content".to_string(), Value::Str("Please create a task to write the report.".to_string()));
            map.insert("author".to_string(), Value::Str("user".to_string()));
            map
        },
    });
    graph.nodes.push(Node {
        id: "user-actor".to_string(),
        node_type: "Actor".to_string(),
        attrs: BTreeMap::new(),
    });

    // Add edges
    graph.edges.push(Edge {
        id: "e1".to_string(),
        edge_type: "HAS_TURN".to_string(),
        src: "thread-1".to_string(),
        dst: "turn-1".to_string(),
        attrs: BTreeMap::new(),
    });
    graph.edges.push(Edge {
        id: "e2".to_string(),
        edge_type: "HAS_MESSAGE".to_string(),
        src: "turn-1".to_string(),
        dst: "msg-1".to_string(),
        attrs: BTreeMap::new(),
    });
    graph.edges.push(Edge {
        id: "e3".to_string(),
        edge_type: "AUTHORED_BY".to_string(),
        src: "msg-1".to_string(),
        dst: "user-actor".to_string(),
        attrs: BTreeMap::new(),
    });

    core.snapshot.graph = graph;
    core.snapshot.metadata.timestamp = fixed_timestamp;
    core
}

fn create_task_rule() -> DpoRule {
    let fixed_timestamp = DateTime::parse_from_rfc3339("2025-11-15T12:00:00Z").unwrap().with_timezone(&Utc);
    DpoRule {
        metadata: RuleMetadata {
            id: "rho_create_task_from_message".to_string(),
            version: "0.1.0".to_string(),
            description: "Creates a new Task node from a user message.".to_string(),
            author: "system".to_string(),
            created_at: fixed_timestamp,
            tags: vec!["task".to_string(), "creation".to_string()],
        },
        left: GraphPattern {
            nodes: vec![
                NodePattern {
                    var: "msg".to_string(),
                    node_type: Some("Message".to_string()),
                    attr_constraints: BTreeMap::new(),
                }
            ],
            edges: vec![],
            constraints: vec![],
        },
        interface: GraphPattern {
            nodes: vec![
                NodePattern {
                    var: "msg".to_string(),
                    node_type: Some("Message".to_string()),
                    attr_constraints: BTreeMap::new(),
                }
            ],
            edges: vec![],
            constraints: vec![],
        },
        right: Graph {
            nodes: vec![
                Node {
                    id: "var:msg".to_string(),
                    node_type: "Message".to_string(),
                    attrs: BTreeMap::new(),
                },
                Node {
                    id: "new:task".to_string(),
                    node_type: "Task".to_string(),
                    attrs: {
                        let mut map = BTreeMap::new();
                        map.insert("title".to_string(), Value::Str("Write the report".to_string()));
                        map.insert("status".to_string(), Value::Str("Pending".to_string()));
                        map
                    },
                }
            ],
            edges: vec![
                Edge {
                    id: "new:edge".to_string(),
                    edge_type: "CREATES_TASK".to_string(),
                    src: "var:msg".to_string(),
                    dst: "new:task".to_string(),
                    attrs: BTreeMap::new(),
                }
            ],
        }
    }
}

#[test]
fn test_v0_apply_create_task_rule() {
    let mut core = create_test_core_with_initial_state();
    let rule = create_task_rule();

    // 1. Validate the rule
    let report = core.validate(&rule).expect("Validation failed");
    assert!(report.is_valid, "Rule should be valid. Errors: {:?}", report.errors);

    // 2. Apply the rule
    let proof = core.apply(&rule).expect("Apply failed");

    // 3. Assert the new state of the graph
    let final_graph = &proof.after_snapshot.unwrap().graph;

    // Should have 5 nodes now (4 initial + 1 new task)
    assert_eq!(final_graph.nodes.len(), 5);
    let task_node = final_graph.nodes.iter().find(|n| n.node_type == "Task").expect("Task node not found");
    assert_eq!(task_node.attrs.get("title"), Some(&Value::Str("Write the report".to_string())));

    // Should have 4 edges now (3 initial + 1 new CREATES_TASK edge)
    assert_eq!(final_graph.edges.len(), 4);
    let creates_task_edge = final_graph.edges.iter().find(|e| e.edge_type == "CREATES_TASK");
    assert!(creates_task_edge.is_some(), "CREATES_TASK edge was not created");
}

#[test]
fn test_mock_oracle_propose_rule() {
    let oracle = MockOracle;
    let core = create_test_core_with_initial_state();
    let snapshot = core.snapshot();
    let command = "Hey, can you create a task to write the report?";

    let rule = oracle.propose_rule(command, &snapshot).expect("Oracle failed to propose a rule");

    assert_eq!(rule.metadata.id, "rho_create_task_from_message_mock");
    assert_eq!(rule.right.nodes.len(), 2);
    assert_eq!(rule.right.edges.len(), 1);

    let task_node = rule.right.nodes.iter().find(|n| n.node_type == "Task").expect("Task node not found in rule");
    assert_eq!(task_node.attrs.get("title"), Some(&Value::Str("Write the report".to_string())));
}

#[test]
fn test_full_round_trip() {
    // 1. Setup
    let mut core = create_test_core_with_initial_state();
    let oracle = MockOracle;
    let command = "Create a task to write the report.";

    // 2. Oracle proposes a rule
    let snapshot = core.snapshot();
    let rule = oracle.propose_rule(command, &snapshot).expect("Oracle should propose a rule");

    // 3. Core validates the rule
    let report = core.validate(&rule).expect("Core should validate the rule");
    assert!(report.is_valid, "Proposed rule should be valid");

    // 4. Core applies the rule
    let proof = core.apply(&rule).expect("Core should apply the rule");

    // 5. Verify the final state
    let final_graph = &proof.after_snapshot.as_ref().unwrap().graph;
    assert_eq!(final_graph.nodes.len(), 5, "Should have 5 nodes after rule application");
    let task_node = final_graph.nodes.iter().find(|n| n.node_type == "Task").expect("Task node should exist");
    assert_eq!(task_node.attrs.get("title"), Some(&Value::Str("Write the report".to_string())));
    assert_eq!(final_graph.edges.len(), 4, "Should have 4 edges after rule application");
    let creates_task_edge = final_graph.edges.iter().find(|e| e.edge_type == "CREATES_TASK").expect("CREATES_TASK edge should exist");
    assert_eq!(creates_task_edge.src, "msg-1");
}

#[test]
fn test_core_is_deterministic() {
    let rule = create_task_rule();

    // Run 1
    let mut core1 = create_test_core_with_initial_state();
    let proof1 = core1.apply(&rule).expect("Run 1: Apply failed");
    let snapshot1 = proof1.after_snapshot.unwrap();

    // Run 2
    let mut core2 = create_test_core_with_initial_state();
    let proof2 = core2.apply(&rule).expect("Run 2: Apply failed");
    let snapshot2 = proof2.after_snapshot.unwrap();

    // To be truly deterministic, the snapshots should be identical.
    // We can compare their serialized form for a robust check.
    let snapshot1_json = serde_json::to_string(&snapshot1).unwrap();
    let snapshot2_json = serde_json::to_string(&snapshot2).unwrap();

    assert_eq!(snapshot1_json, snapshot2_json, "Snapshots from two identical runs should be identical");
}