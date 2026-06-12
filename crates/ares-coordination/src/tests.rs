use super::*;
use ares_agent_runtime::models::*;

// ═══════════════════════════════════════════════════════════════════
// Phase 1 — Organization Tests (~30)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_agent_node_creation() {
    let id = AgentId::new();
    let node = organization::AgentNode::new(id, AgentRole::Coder);
    assert_eq!(node.agent_id, id);
    assert_eq!(node.role, AgentRole::Coder);
    assert!(node.is_root());
    assert!(node.is_leaf());
}

#[test]
fn test_agent_node_with_team() {
    let node =
        organization::AgentNode::new(AgentId::new(), AgentRole::Tester).with_team(TeamId::new());
    assert!(node.team_id.is_some());
}

#[test]
fn test_agent_node_with_parent() {
    let parent_id = AgentId::new();
    let node =
        organization::AgentNode::new(AgentId::new(), AgentRole::Coder).with_parent(parent_id);
    assert!(!node.is_root());
    assert_eq!(node.parent_id, Some(parent_id));
}

#[test]
fn test_agent_node_children() {
    let mut node = organization::AgentNode::new(AgentId::new(), AgentRole::CEO);
    let child1 = AgentId::new();
    let child2 = AgentId::new();
    node.add_child(child1);
    node.add_child(child2);
    assert_eq!(node.child_count(), 2);
    assert!(!node.is_leaf());
    assert!(node.remove_child(&child1));
    assert_eq!(node.child_count(), 1);
}

#[test]
fn test_agent_node_capabilities() {
    let node = organization::AgentNode::new(AgentId::new(), AgentRole::Coder)
        .with_capabilities(vec!["code_generation".into(), "testing".into()]);
    assert!(node.has_capability("code_generation"));
    assert!(!node.has_capability("security_scan"));
}

#[test]
fn test_agent_node_status() {
    let mut node = organization::AgentNode::new(AgentId::new(), AgentRole::Coder);
    assert_eq!(node.status, organization::node::NodeStatus::Active);
    node.set_status(organization::node::NodeStatus::Busy);
    assert_eq!(node.status, organization::node::NodeStatus::Busy);
}

#[test]
fn test_agent_team_creation() {
    let team = organization::team::AgentTeam::new("engineering", "Build the feature");
    assert_eq!(team.name, "engineering");
    assert_eq!(team.goal, "Build the feature");
    assert_eq!(team.member_count(), 0);
    assert!(!team.is_complete());
}

#[test]
fn test_agent_team_members() {
    let mut team = organization::team::AgentTeam::new("test", "test");
    let agent1 = AgentId::new();
    let agent2 = AgentId::new();
    team.add_member(agent1, AgentRole::Coder);
    team.add_member(agent2, AgentRole::Tester);
    team.set_leader(agent1);
    assert_eq!(team.member_count(), 2);
    assert!(team.has_member(&agent1));
    assert!(team.is_complete());
}

#[test]
fn test_agent_team_get_by_role() {
    let mut team = organization::team::AgentTeam::new("test", "test");
    let c1 = AgentId::new();
    let c2 = AgentId::new();
    let t1 = AgentId::new();
    team.add_member(c1, AgentRole::Coder);
    team.add_member(c2, AgentRole::Coder);
    team.add_member(t1, AgentRole::Tester);
    let coders = team.get_members_by_role(&AgentRole::Coder);
    assert_eq!(coders.len(), 2);
}

#[test]
fn test_agent_team_remove_member() {
    let mut team = organization::team::AgentTeam::new("test", "test");
    let agent = AgentId::new();
    team.add_member(agent, AgentRole::Coder);
    team.set_leader(agent);
    assert!(team.remove_member(&agent));
    assert!(team.leader_id.is_none());
    assert_eq!(team.member_count(), 0);
}

#[test]
fn test_agent_team_strategy() {
    let team = organization::team::AgentTeam::new("test", "test")
        .with_strategy(organization::team::TeamStrategy::Pipeline);
    assert_eq!(team.strategy, organization::team::TeamStrategy::Pipeline);
}

#[test]
fn test_agent_team_disband() {
    let mut team = organization::team::AgentTeam::new("test", "test");
    team.add_member(AgentId::new(), AgentRole::Coder);
    team.disband();
    assert_eq!(team.status, organization::team::TeamStatus::Disbanded);
    assert_eq!(team.member_count(), 0);
}

#[test]
fn test_hierarchy_creation() {
    let hierarchy = organization::hierarchy::AgentHierarchy::new("test org");
    assert_eq!(hierarchy.name, "test org");
    assert_eq!(hierarchy.node_count(), 0);
    assert!(hierarchy.get_root().is_none());
}

#[test]
fn test_hierarchy_add_node() {
    let mut h = organization::hierarchy::AgentHierarchy::new("test");
    let id = AgentId::new();
    h.add_node(organization::AgentNode::new(id, AgentRole::CEO));
    assert_eq!(h.node_count(), 1);
    assert_eq!(h.root_id, Some(id));
}

#[test]
fn test_hierarchy_parent_child() {
    let mut h = organization::hierarchy::AgentHierarchy::new("test");
    let parent = AgentId::new();
    let child = AgentId::new();
    h.add_node(organization::AgentNode::new(parent, AgentRole::CEO));
    h.add_node(organization::AgentNode::new(child, AgentRole::Coder));
    h.set_parent(child, parent).unwrap();

    let children = h.get_children(&parent);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].agent_id, child);
}

#[test]
fn test_hierarchy_depth() {
    let mut h = organization::hierarchy::AgentHierarchy::new("test");
    let a = AgentId::new();
    let b = AgentId::new();
    let c = AgentId::new();
    h.add_node(organization::AgentNode::new(a, AgentRole::CEO));
    h.add_node(organization::AgentNode::new(b, AgentRole::Architect));
    h.add_node(organization::AgentNode::new(c, AgentRole::Coder));
    h.set_parent(b, a).unwrap();
    h.set_parent(c, b).unwrap();

    assert_eq!(h.get_depth(&a), 0);
    assert_eq!(h.get_depth(&b), 1);
    assert_eq!(h.get_depth(&c), 2);
}

#[test]
fn test_hierarchy_descendants() {
    let mut h = organization::hierarchy::AgentHierarchy::new("test");
    let root = AgentId::new();
    let mid = AgentId::new();
    let leaf = AgentId::new();
    h.add_node(organization::AgentNode::new(root, AgentRole::CEO));
    h.add_node(organization::AgentNode::new(mid, AgentRole::Architect));
    h.add_node(organization::AgentNode::new(leaf, AgentRole::Coder));
    h.set_parent(mid, root).unwrap();
    h.set_parent(leaf, mid).unwrap();

    let descendants = h.get_descendants(&root);
    assert_eq!(descendants.len(), 2);
}

#[test]
fn test_hierarchy_nodes_by_role() {
    let mut h = organization::hierarchy::AgentHierarchy::new("test");
    h.add_node(organization::AgentNode::new(
        AgentId::new(),
        AgentRole::Coder,
    ));
    h.add_node(organization::AgentNode::new(
        AgentId::new(),
        AgentRole::Coder,
    ));
    h.add_node(organization::AgentNode::new(
        AgentId::new(),
        AgentRole::Tester,
    ));

    assert_eq!(h.get_nodes_by_role(&AgentRole::Coder).len(), 2);
    assert_eq!(h.get_nodes_by_role(&AgentRole::Tester).len(), 1);
}

#[test]
fn test_hierarchy_remove_node() {
    let mut h = organization::hierarchy::AgentHierarchy::new("test");
    let root = AgentId::new();
    let mid = AgentId::new();
    let leaf = AgentId::new();
    h.add_node(organization::AgentNode::new(root, AgentRole::CEO));
    h.add_node(organization::AgentNode::new(mid, AgentRole::Architect));
    h.add_node(organization::AgentNode::new(leaf, AgentRole::Coder));
    h.set_parent(mid, root).unwrap();
    h.set_parent(leaf, mid).unwrap();

    h.remove_node(&mid).unwrap();
    assert_eq!(h.node_count(), 2);
    // leaf should now be child of root
    assert_eq!(h.get_children(&root).len(), 1);
}

#[test]
fn test_organization_builder() {
    let h = organization::OrganizationBuilder::new("test org")
        .with_leader(AgentRole::CEO)
        .add_team("eng", &[AgentRole::Coder, AgentRole::Tester])
        .add_team("research", &[AgentRole::Researcher])
        .build();

    assert!(h.node_count() >= 4); // CEO + 2 eng + 1 research
    assert_eq!(h.team_count(), 2);
    assert!(h.get_root().is_some());
}

#[test]
fn test_organization_builder_with_strategy() {
    let h = organization::OrganizationBuilder::new("test")
        .with_leader(AgentRole::Architect)
        .add_team_with_strategy(
            "pipeline",
            &[AgentRole::Coder, AgentRole::Reviewer],
            organization::team::TeamStrategy::Pipeline,
        )
        .build();

    assert!(h.team_count() >= 1);
}

#[test]
fn test_standard_engineering_org() {
    let h = organization::builder::build_standard_engineering_org();
    assert!(h.node_count() >= 9); // CEO + 3 planning + 3 engineering + 2 ops
    assert_eq!(h.team_count(), 3);
}

#[test]
fn test_research_org() {
    let h = organization::builder::build_research_org();
    assert!(h.node_count() >= 5);
    assert_eq!(h.team_count(), 2);
}

#[test]
fn test_assignment_creation() {
    let a = organization::assignment::AgentAssignment::new(
        AgentId::new(),
        TaskId::new(),
        AgentRole::Coder,
    );
    assert_eq!(
        a.status,
        organization::assignment::AssignmentStatus::Pending
    );
    assert!(!a.is_active());
    assert!(!a.is_terminal());
}

#[test]
fn test_assignment_lifecycle() {
    let mut a = organization::assignment::AgentAssignment::new(
        AgentId::new(),
        TaskId::new(),
        AgentRole::Coder,
    );
    a.activate();
    assert!(a.is_active());
    a.complete();
    assert!(a.is_terminal());
}

#[test]
fn test_assignment_reassign() {
    let mut a = organization::assignment::AgentAssignment::new(
        AgentId::new(),
        TaskId::new(),
        AgentRole::Coder,
    );
    let new_agent = AgentId::new();
    a.reassign(new_agent);
    assert_eq!(a.agent_id, new_agent);
}

#[test]
fn test_assignment_priority() {
    let a = organization::assignment::AgentAssignment::new(
        AgentId::new(),
        TaskId::new(),
        AgentRole::Coder,
    )
    .with_priority(organization::assignment::AssignmentPriority::Critical);
    assert_eq!(
        a.priority,
        organization::assignment::AssignmentPriority::Critical
    );
}

// ═══════════════════════════════════════════════════════════════════
// Phase 2 — Messaging Tests (~35)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_message_direct() {
    let from = AgentId::new();
    let to = AgentId::new();
    let msg = messaging::Message::direct(from, to, "hello", "world");
    assert_eq!(msg.from, from);
    assert_eq!(msg.to, Some(to));
    assert_eq!(msg.msg_type, messaging::MessageType::Direct);
}

#[test]
fn test_message_broadcast() {
    let from = AgentId::new();
    let msg = messaging::Message::broadcast(from, "announcement", "test");
    assert!(msg.is_broadcast());
    assert!(msg.to.is_none());
}

#[test]
fn test_message_team() {
    let from = AgentId::new();
    let team = TeamId::new();
    let msg = messaging::Message::team(from, team, "team update", "details");
    assert!(msg.is_team_message());
}

#[test]
fn test_message_request_response() {
    let from = AgentId::new();
    let to = AgentId::new();
    let req = messaging::Message::request(from, to, "compute", "data");
    assert!(req.ack_required);
    let resp = messaging::Message::response(to, from, req.id, "result");
    assert!(matches!(resp.msg_type, messaging::MessageType::Response(_)));
}

#[test]
fn test_message_priority() {
    let msg = messaging::Message::direct(AgentId::new(), AgentId::new(), "test", "test")
        .with_priority(messaging::Priority::Critical);
    assert_eq!(msg.priority, messaging::Priority::Critical);
}

#[test]
fn test_message_ttl() {
    let msg =
        messaging::Message::direct(AgentId::new(), AgentId::new(), "test", "test").with_ttl(0);
    // TTL of 0 means immediately expired
    assert!(msg.is_expired());
}

#[test]
fn test_message_ttl_not_expired() {
    let msg =
        messaging::Message::direct(AgentId::new(), AgentId::new(), "test", "test").with_ttl(3600);
    assert!(!msg.is_expired());
}

#[test]
fn test_message_no_ttl() {
    let msg = messaging::Message::direct(AgentId::new(), AgentId::new(), "test", "test");
    assert!(!msg.is_expired());
}

#[test]
fn test_message_with_conversation() {
    let conv = ConversationId::new();
    let msg = messaging::Message::direct(AgentId::new(), AgentId::new(), "test", "test")
        .with_conversation(conv);
    assert_eq!(msg.conversation_id, Some(conv));
}

#[test]
fn test_message_with_mission() {
    let mid = MissionId::new();
    let msg = messaging::Message::direct(AgentId::new(), AgentId::new(), "test", "test")
        .with_mission(mid);
    assert_eq!(msg.mission_id, Some(mid));
}

#[tokio::test]
async fn test_message_bus_register() {
    let bus = messaging::MessageBus::new();
    let id = AgentId::new();
    let _rx = bus.register(id).await;
    assert_eq!(bus.agent_count().await, 1);
    assert!(bus.is_registered(&id).await);
}

#[tokio::test]
async fn test_message_bus_unregister() {
    let bus = messaging::MessageBus::new();
    let id = AgentId::new();
    let _rx = bus.register(id).await;
    bus.unregister(&id).await;
    assert_eq!(bus.agent_count().await, 0);
}

#[tokio::test]
async fn test_message_bus_send() {
    let bus = messaging::MessageBus::new();
    let from = AgentId::new();
    let to = AgentId::new();
    let _rx_from = bus.register(from).await;
    let mut rx_to = bus.register(to).await;

    let msg = messaging::Message::direct(from, to, "hello", "world");
    bus.send(msg).await.unwrap();

    let received = rx_to.recv().await.unwrap();
    assert_eq!(received.subject, "hello");
}

#[tokio::test]
async fn test_message_bus_send_unregistered() {
    let bus = messaging::MessageBus::new();
    let msg = messaging::Message::direct(AgentId::new(), AgentId::new(), "test", "test");
    assert!(bus.send(msg).await.is_err());
}

#[tokio::test]
async fn test_message_bus_broadcast() {
    let bus = messaging::MessageBus::new();
    let sender = AgentId::new();
    let r1 = AgentId::new();
    let r2 = AgentId::new();
    let _rx_s = bus.register(sender).await;
    let mut rx1 = bus.register(r1).await;
    let mut rx2 = bus.register(r2).await;

    let msg = messaging::Message::broadcast(sender, "announcement", "hello all");
    let delivered = bus.broadcast(msg).await.unwrap();
    assert_eq!(delivered, 2);

    let m1 = rx1.recv().await.unwrap();
    let m2 = rx2.recv().await.unwrap();
    assert_eq!(m1.subject, "announcement");
    assert_eq!(m2.subject, "announcement");
}

#[tokio::test]
async fn test_message_bus_team_send() {
    let bus = messaging::MessageBus::new();
    let sender = AgentId::new();
    let member1 = AgentId::new();
    let member2 = AgentId::new();
    let outsider = AgentId::new();
    let _rx_s = bus.register(sender).await;
    let mut rx1 = bus.register(member1).await;
    let mut rx2 = bus.register(member2).await;
    let _rx_o = bus.register(outsider).await;

    let msg = messaging::Message::team(sender, TeamId::new(), "team msg", "data");
    let delivered = bus.send_to_team(msg, &[member1, member2]).await.unwrap();
    assert_eq!(delivered, 2);

    let _ = rx1.recv().await.unwrap();
    let _ = rx2.recv().await.unwrap();
}

#[tokio::test]
async fn test_message_bus_log() {
    let bus = messaging::MessageBus::new();
    let from = AgentId::new();
    let to = AgentId::new();
    let _rx_f = bus.register(from).await;
    let _rx_t = bus.register(to).await;

    let msg = messaging::Message::direct(from, to, "test", "test");
    bus.send(msg).await.unwrap();

    assert_eq!(bus.message_count().await, 1);
    let log = bus.get_message_log().await;
    assert_eq!(log.len(), 1);
}

#[tokio::test]
async fn test_message_bus_clear_log() {
    let bus = messaging::MessageBus::new();
    let from = AgentId::new();
    let to = AgentId::new();
    let _rx_f = bus.register(from).await;
    let _rx_t = bus.register(to).await;

    bus.send(messaging::Message::direct(from, to, "test", "test"))
        .await
        .unwrap();
    bus.clear_log().await;
    assert_eq!(bus.message_count().await, 0);
}

#[test]
fn test_conversation_creation() {
    let agent = AgentId::new();
    let conv = messaging::Conversation::new("topic", agent);
    assert_eq!(conv.topic, "topic");
    assert!(conv.has_participant(&agent));
    assert!(conv.is_active());
    assert_eq!(conv.message_count(), 0);
}

#[test]
fn test_conversation_add_message() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let mut conv = messaging::conversation::Conversation::new("test", a1);
    let msg = messaging::Message::direct(a1, a2, "hello", "world");
    conv.add_message(msg);
    assert_eq!(conv.message_count(), 1);
    assert!(conv.has_participant(&a2));
    assert_eq!(conv.participant_count(), 2);
}

#[test]
fn test_conversation_resolve() {
    let mut conv = messaging::conversation::Conversation::new("test", AgentId::new());
    conv.resolve();
    assert!(!conv.is_active());
}

#[test]
fn test_conversation_manager() {
    let mut mgr = messaging::conversation::ConversationManager::new();
    let agent = AgentId::new();
    let id = mgr.create("topic", agent);
    assert_eq!(mgr.conversation_count(), 1);
    assert!(mgr.get(&id).is_some());
    assert_eq!(mgr.get_active_conversations().len(), 1);
    assert_eq!(mgr.get_conversations_for_agent(&agent).len(), 1);
}

#[test]
fn test_conversation_manager_add_message() {
    let mut mgr = messaging::conversation::ConversationManager::new();
    let agent = AgentId::new();
    let id = mgr.create("topic", agent);
    let msg = messaging::Message::broadcast(agent, "test", "test");
    mgr.add_message(&id, msg).unwrap();
    assert_eq!(mgr.get(&id).unwrap().message_count(), 1);
}

#[test]
fn test_priority_ordering() {
    assert!(messaging::Priority::Critical > messaging::Priority::High);
    assert!(messaging::Priority::High > messaging::Priority::Normal);
    assert!(messaging::Priority::Normal > messaging::Priority::Low);
}

// ═══════════════════════════════════════════════════════════════════
// Phase 3 — Shared Memory Tests (~25)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_shared_fact_creation() {
    let fact = shared_memory::SharedFact::new(
        AgentId::new(),
        shared_memory::FactCategory::Fact,
        "key",
        "value",
    );
    assert!(fact.is_active());
    assert_eq!(fact.confidence, 1.0);
}

#[test]
fn test_shared_fact_with_confidence() {
    let fact = shared_memory::SharedFact::new(
        AgentId::new(),
        shared_memory::FactCategory::Assumption,
        "key",
        "value",
    )
    .with_confidence(0.7);
    assert!((fact.confidence - 0.7).abs() < f64::EPSILON);
}

#[test]
fn test_shared_fact_confidence_clamp() {
    let fact = shared_memory::SharedFact::new(
        AgentId::new(),
        shared_memory::FactCategory::Fact,
        "key",
        "value",
    )
    .with_confidence(1.5);
    assert!((fact.confidence - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_shared_fact_retract() {
    let mut fact = shared_memory::SharedFact::new(
        AgentId::new(),
        shared_memory::FactCategory::Fact,
        "key",
        "value",
    );
    fact.retract();
    assert!(!fact.is_active());
}

#[test]
fn test_shared_fact_update_value() {
    let mut fact = shared_memory::SharedFact::new(
        AgentId::new(),
        shared_memory::FactCategory::Observation,
        "key",
        "old",
    );
    fact.update_value("new");
    assert_eq!(fact.value, "new");
}

#[tokio::test]
async fn test_workspace_publish_and_query() {
    let ws = shared_memory::SharedWorkspace::new();
    let agent = AgentId::new();
    let fact = shared_memory::SharedFact::new(
        agent,
        shared_memory::FactCategory::Fact,
        "test_key",
        "test_value",
    );
    ws.publish_fact(fact).await;

    let facts = ws
        .query_by_category(&shared_memory::FactCategory::Fact)
        .await;
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].key, "test_key");
}

#[tokio::test]
async fn test_workspace_query_by_key_prefix() {
    let ws = shared_memory::SharedWorkspace::new();
    let agent = AgentId::new();
    ws.publish_fact(shared_memory::SharedFact::new(
        agent,
        shared_memory::FactCategory::Fact,
        "config.timeout",
        "30",
    ))
    .await;
    ws.publish_fact(shared_memory::SharedFact::new(
        agent,
        shared_memory::FactCategory::Fact,
        "config.retries",
        "3",
    ))
    .await;
    ws.publish_fact(shared_memory::SharedFact::new(
        agent,
        shared_memory::FactCategory::Fact,
        "data.size",
        "1000",
    ))
    .await;

    let config_facts = ws.query_by_key_prefix("config.").await;
    assert_eq!(config_facts.len(), 2);
}

#[tokio::test]
async fn test_workspace_query_by_author() {
    let ws = shared_memory::SharedWorkspace::new();
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    ws.publish_fact(shared_memory::SharedFact::new(
        a1,
        shared_memory::FactCategory::Fact,
        "k1",
        "v1",
    ))
    .await;
    ws.publish_fact(shared_memory::SharedFact::new(
        a2,
        shared_memory::FactCategory::Fact,
        "k2",
        "v2",
    ))
    .await;
    ws.publish_fact(shared_memory::SharedFact::new(
        a1,
        shared_memory::FactCategory::Fact,
        "k3",
        "v3",
    ))
    .await;

    let a1_facts = ws.query_by_author(&a1).await;
    assert_eq!(a1_facts.len(), 2);
}

#[tokio::test]
async fn test_workspace_retract_fact() {
    let ws = shared_memory::SharedWorkspace::new();
    let fact =
        shared_memory::SharedFact::new(AgentId::new(), shared_memory::FactCategory::Fact, "k", "v");
    let id = ws.publish_fact(fact).await;
    ws.retract_fact(&id).await.unwrap();

    assert_eq!(ws.active_fact_count().await, 0);
    assert_eq!(ws.fact_count().await, 1); // still stored
}

#[tokio::test]
async fn test_workspace_update_fact() {
    let ws = shared_memory::SharedWorkspace::new();
    let fact = shared_memory::SharedFact::new(
        AgentId::new(),
        shared_memory::FactCategory::Fact,
        "k",
        "old",
    );
    let id = ws.publish_fact(fact).await;
    ws.update_fact(&id, "new").await.unwrap();

    let updated = ws.get_fact(&id).await.unwrap();
    assert_eq!(updated.value, "new");
}

#[tokio::test]
async fn test_workspace_snapshot() {
    let ws = shared_memory::SharedWorkspace::new();
    let agent = AgentId::new();
    ws.publish_fact(shared_memory::SharedFact::new(
        agent,
        shared_memory::FactCategory::Fact,
        "k1",
        "v1",
    ))
    .await;
    ws.publish_fact(shared_memory::SharedFact::new(
        agent,
        shared_memory::FactCategory::Plan,
        "k2",
        "v2",
    ))
    .await;

    let snapshot = ws.snapshot().await;
    assert_eq!(snapshot.len(), 2);
}

#[tokio::test]
async fn test_workspace_clear() {
    let ws = shared_memory::SharedWorkspace::new();
    ws.publish_fact(shared_memory::SharedFact::new(
        AgentId::new(),
        shared_memory::FactCategory::Fact,
        "k",
        "v",
    ))
    .await;
    ws.clear().await;
    assert_eq!(ws.fact_count().await, 0);
}

#[tokio::test]
async fn test_workspace_for_mission() {
    let mid = MissionId::new();
    let ws = shared_memory::SharedWorkspace::for_mission(mid);
    assert_eq!(ws.mission_id, Some(mid));
}

#[tokio::test]
async fn test_workspace_for_team() {
    let tid = TeamId::new();
    let ws = shared_memory::SharedWorkspace::for_team(tid);
    assert_eq!(ws.team_id, Some(tid));
}

// ═══════════════════════════════════════════════════════════════════
// Phase 4 — Resource Manager Tests (~20)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_resource_requirements_presets() {
    let min = resource_manager::models::ResourceRequirements::minimal();
    let std = resource_manager::models::ResourceRequirements::standard();
    let heavy = resource_manager::models::ResourceRequirements::heavy();
    assert!(min.cpu_slots < std.cpu_slots);
    assert!(std.cpu_slots < heavy.cpu_slots);
}

#[test]
fn test_resource_utilization_calculations() {
    let util = resource_manager::models::ResourceUtilization {
        cpu_used: 8,
        cpu_total: 16,
        memory_used_mb: 2048,
        memory_total_mb: 4096,
        gpu_used: 0,
        gpu_total: 0,
        tokens_consumed: 500000,
        token_budget: 1000000,
        tool_slots_used: 4,
        tool_slots_total: 8,
        network_slots_used: 0,
        network_slots_total: 4,
        active_reservations: 5,
    };
    assert!((util.cpu_utilization() - 0.5).abs() < f64::EPSILON);
    assert!((util.memory_utilization() - 0.5).abs() < f64::EPSILON);
    assert!((util.token_utilization() - 0.5).abs() < f64::EPSILON);
    assert!(!util.is_overloaded());
}

#[test]
fn test_resource_utilization_overloaded() {
    let util = resource_manager::models::ResourceUtilization {
        cpu_used: 15,
        cpu_total: 16,
        memory_used_mb: 100,
        memory_total_mb: 4096,
        gpu_used: 0,
        gpu_total: 0,
        tokens_consumed: 0,
        token_budget: 1000000,
        tool_slots_used: 0,
        tool_slots_total: 8,
        network_slots_used: 0,
        network_slots_total: 4,
        active_reservations: 0,
    };
    assert!(util.is_overloaded()); // CPU > 90%
}

#[test]
fn test_resource_check_availability() {
    let mgr = resource_manager::CoordinationResourceManager::default();
    let req = resource_manager::models::ResourceRequirements::minimal();
    assert!(mgr.check_availability(&req));
}

#[tokio::test]
async fn test_resource_reserve_and_release() {
    let mgr = resource_manager::CoordinationResourceManager::default();
    let req = resource_manager::models::ResourceRequirements::standard();
    let id = mgr.reserve(TaskId::new(), req).await.unwrap();
    assert_eq!(mgr.reservation_count().await, 1);
    mgr.release(&id).await.unwrap();
    assert_eq!(mgr.reservation_count().await, 0);
}

#[tokio::test]
async fn test_resource_exhaustion() {
    let cap = resource_manager::models::ResourceCapacity {
        cpu_slots: 2,
        memory_mb: 128,
        gpu_slots: 0,
        token_budget: 1000,
        tool_slots: 1,
        network_slots: 1,
    };
    let mgr = resource_manager::CoordinationResourceManager::new(cap);
    let req = resource_manager::models::ResourceRequirements {
        cpu_slots: 2,
        memory_mb: 128,
        gpu_slots: 0,
        token_budget: 1000,
        tool_slots: 1,
        network_slots: 1,
    };
    let _id = mgr.reserve(TaskId::new(), req.clone()).await.unwrap();
    assert!(mgr.reserve(TaskId::new(), req).await.is_err());
}

#[test]
fn test_resource_consume_tokens() {
    let mgr = resource_manager::CoordinationResourceManager::default();
    assert!(mgr.consume_tokens(100).is_ok());
}

#[test]
fn test_resource_token_budget_exceeded() {
    let cap = resource_manager::models::ResourceCapacity {
        cpu_slots: 16,
        memory_mb: 4096,
        gpu_slots: 0,
        token_budget: 100,
        tool_slots: 8,
        network_slots: 4,
    };
    let mgr = resource_manager::CoordinationResourceManager::new(cap);
    assert!(mgr.consume_tokens(50).is_ok());
    assert!(mgr.consume_tokens(51).is_err());
}

#[tokio::test]
async fn test_resource_utilization_snapshot() {
    let mgr = resource_manager::CoordinationResourceManager::default();
    let util = mgr.get_utilization().await;
    assert_eq!(util.active_reservations, 0);
    assert!((util.cpu_utilization()).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════
// Phase 5 — Governor Tests (~25)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_governor_delegation_allow() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_delegation(1).await;
    assert!(decision.is_allowed());
}

#[tokio::test]
async fn test_governor_delegation_deny() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_delegation(10).await;
    assert!(decision.is_denied());
}

#[tokio::test]
async fn test_governor_delegation_throttle() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_delegation(5).await; // max is 5
    assert!(matches!(decision, governor::GovernorDecision::Throttle(_)));
}

#[tokio::test]
async fn test_governor_swarm_allow() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_swarm_launch(5).await;
    assert!(decision.is_allowed());
}

#[tokio::test]
async fn test_governor_swarm_deny() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_swarm_launch(20).await;
    assert!(decision.is_denied());
}

#[tokio::test]
async fn test_governor_debate_allow() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_debate(2).await;
    assert!(decision.is_allowed());
}

#[tokio::test]
async fn test_governor_debate_deny() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_debate(5).await; // max is 5
    assert!(decision.is_denied());
}

#[tokio::test]
async fn test_governor_consensus_allow() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_consensus(1).await;
    assert!(decision.is_allowed());
}

#[tokio::test]
async fn test_governor_consensus_deny() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_consensus(3).await; // max is 3
    assert!(decision.is_denied());
}

#[tokio::test]
async fn test_governor_cost_allow() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_cost(50.0).await;
    assert!(decision.is_allowed());
}

#[tokio::test]
async fn test_governor_cost_deny() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_cost(150.0).await; // max is 100
    assert!(decision.is_denied());
}

#[tokio::test]
async fn test_governor_concurrent_tasks() {
    let gov = governor::SafetyGovernor::default();
    let decision = gov.check_concurrent_tasks().await;
    assert!(decision.is_allowed());
}

#[tokio::test]
async fn test_governor_task_tracking() {
    let gov = governor::SafetyGovernor::default();
    gov.task_started();
    gov.task_started();
    assert_eq!(gov.active_task_count(), 2);
    gov.task_completed();
    assert_eq!(gov.active_task_count(), 1);
}

#[tokio::test]
async fn test_governor_cost_recording() {
    let gov = governor::SafetyGovernor::default();
    gov.record_cost(25.0).await;
    gov.record_cost(30.0).await;
    assert!((gov.total_cost().await - 55.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_governor_decision_log() {
    let gov = governor::SafetyGovernor::default();
    gov.check_delegation(1).await;
    gov.check_swarm_launch(3).await;
    assert_eq!(gov.decision_count().await, 2);
}

#[tokio::test]
async fn test_governor_org_depth() {
    let gov = governor::SafetyGovernor::default();
    let allow = gov.check_org_depth(3).await;
    assert!(allow.is_allowed());
    let deny = gov.check_org_depth(10).await;
    assert!(deny.is_denied());
}

#[tokio::test]
async fn test_governor_custom_rules() {
    let rules = governor::GovernorRules {
        max_delegation_depth: 2,
        max_swarm_size: 3,
        max_debate_rounds: 2,
        ..Default::default()
    };
    let gov = governor::SafetyGovernor::new(rules);
    assert!(gov.check_delegation(3).await.is_denied());
    assert!(gov.check_swarm_launch(4).await.is_denied());
    assert!(gov.check_debate(2).await.is_denied());
}

// ═══════════════════════════════════════════════════════════════════
// Phase 6 — Delegation Tests (~30)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_delegation_assign() {
    let engine = delegation::DelegationEngine::new();
    let id = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            0,
            "test",
            None,
        )
        .await
        .unwrap();
    assert_eq!(engine.delegation_count().await, 1);
    let record = engine.get_delegation(&id).await.unwrap();
    assert_eq!(record.status, delegation::DelegationStatus::Pending);
}

#[tokio::test]
async fn test_delegation_with_governor() {
    let gov = governor::SafetyGovernor::default();
    let engine = delegation::DelegationEngine::new();
    // Within limits
    let result = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            1,
            "test",
            Some(&gov),
        )
        .await;
    assert!(result.is_ok());
    // Over limit
    let result = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            10,
            "test",
            Some(&gov),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delegation_reassign() {
    let engine = delegation::DelegationEngine::new();
    let id = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            0,
            "test",
            None,
        )
        .await
        .unwrap();
    let new_id = engine.reassign_task(&id, AgentId::new()).await.unwrap();
    assert_ne!(id, new_id);
    assert_eq!(engine.delegation_count().await, 2);
}

#[tokio::test]
async fn test_delegation_complete() {
    let engine = delegation::DelegationEngine::new();
    let id = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            0,
            "test",
            None,
        )
        .await
        .unwrap();
    engine.complete_delegation(&id, "done").await.unwrap();
    let record = engine.get_delegation(&id).await.unwrap();
    assert_eq!(record.status, delegation::DelegationStatus::Completed);
}

#[tokio::test]
async fn test_delegation_fail() {
    let engine = delegation::DelegationEngine::new();
    let id = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            0,
            "test",
            None,
        )
        .await
        .unwrap();
    engine.fail_delegation(&id, "error").await.unwrap();
    let record = engine.get_delegation(&id).await.unwrap();
    assert_eq!(record.status, delegation::DelegationStatus::Failed);
}

#[tokio::test]
async fn test_delegation_escalate() {
    let engine = delegation::DelegationEngine::new();
    let id = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            0,
            "test",
            None,
        )
        .await
        .unwrap();
    engine.escalate_task(&id, "too complex").await.unwrap();
    let record = engine.get_delegation(&id).await.unwrap();
    assert_eq!(record.status, delegation::DelegationStatus::Escalated);
}

#[tokio::test]
async fn test_delegation_merge_results() {
    let engine = delegation::DelegationEngine::new();
    let id1 = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            0,
            "sub1",
            None,
        )
        .await
        .unwrap();
    let id2 = engine
        .assign_task(
            TaskId::new(),
            AgentId::new(),
            AgentId::new(),
            0,
            "sub2",
            None,
        )
        .await
        .unwrap();
    engine.complete_delegation(&id1, "result_1").await.unwrap();
    engine.complete_delegation(&id2, "result_2").await.unwrap();

    let merged = engine.merge_results(&[id1, id2]).await.unwrap();
    assert!(merged.contains("result_1"));
    assert!(merged.contains("result_2"));
}

#[test]
fn test_task_splitter_simple() {
    let splitter = delegation::TaskSplitter::new();
    let result = splitter.analyze("simple task", 0.2);
    assert_eq!(result.strategy, delegation::SplitStrategy::Unsplittable);
}

#[test]
fn test_task_splitter_medium() {
    let splitter = delegation::TaskSplitter::new();
    let result = splitter.analyze("medium task", 0.6);
    assert_eq!(result.strategy, delegation::SplitStrategy::Parallel);
    assert_eq!(result.sub_tasks.len(), 2);
}

#[test]
fn test_task_splitter_complex() {
    let splitter = delegation::TaskSplitter::new();
    let result = splitter.analyze("complex task", 0.9);
    assert_eq!(result.strategy, delegation::SplitStrategy::Pipeline);
    assert_eq!(result.sub_tasks.len(), 3);
}

#[test]
fn test_task_splitter_sequential() {
    let splitter = delegation::TaskSplitter::new();
    let result = splitter.analyze("light task", 0.4);
    assert_eq!(result.strategy, delegation::SplitStrategy::Sequential);
}

// ═══════════════════════════════════════════════════════════════════
// Phase 7 — Reputation Tests (~30)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_reputation_initial() {
    let rep = reputation::models::AgentReputation::new();
    assert!((rep.success_rate - 0.5).abs() < f64::EPSILON);
    assert_eq!(rep.task_count, 0);
}

#[test]
fn test_reputation_composite_score() {
    let rep = reputation::models::AgentReputation {
        success_rate: 0.9,
        avg_latency_ms: 100.0,
        reliability: 0.95,
        cost_efficiency: 0.8,
        quality_score: 0.85,
        task_count: 10,
        updated_at: 0,
    };
    let score = rep.composite_score();
    assert!(score > 0.0 && score <= 1.0);
}

#[test]
fn test_reputation_tracker_record() {
    let mut tracker = reputation::ReputationTracker::new();
    let agent = AgentId::new();
    let outcome = reputation::models::TaskOutcome {
        success: true,
        latency_ms: 100.0,
        cost: 5.0,
        quality: 0.9,
    };
    tracker.record_outcome(agent, &outcome);
    assert!(tracker.has_agent(&agent));
    let rep = tracker.get_reputation(&agent).unwrap();
    assert_eq!(rep.task_count, 1);
    assert!((rep.success_rate - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_reputation_tracker_ema_convergence() {
    let mut tracker = reputation::ReputationTracker::new();
    let agent = AgentId::new();
    for _ in 0..20 {
        tracker.record_outcome(
            agent,
            &reputation::models::TaskOutcome {
                success: false,
                latency_ms: 5000.0,
                cost: 100.0,
                quality: 0.1,
            },
        );
    }
    let rep = tracker.get_reputation(&agent).unwrap();
    assert!(rep.success_rate < 0.1);
    assert!(rep.quality_score < 0.2);
}

#[test]
fn test_reputation_tracker_ranking() {
    let mut tracker = reputation::ReputationTracker::new();
    let good = AgentId::new();
    let bad = AgentId::new();
    for _ in 0..5 {
        tracker.record_outcome(
            good,
            &reputation::models::TaskOutcome {
                success: true,
                latency_ms: 100.0,
                cost: 1.0,
                quality: 0.95,
            },
        );
        tracker.record_outcome(
            bad,
            &reputation::models::TaskOutcome {
                success: false,
                latency_ms: 5000.0,
                cost: 50.0,
                quality: 0.1,
            },
        );
    }
    let ranked = tracker.rank_agents(&AgentRole::Coder);
    assert_eq!(ranked[0].0, good);
}

#[test]
fn test_reputation_decay() {
    let mut tracker = reputation::ReputationTracker::new();
    let agent = AgentId::new();
    tracker.record_outcome(
        agent,
        &reputation::models::TaskOutcome {
            success: true,
            latency_ms: 100.0,
            cost: 1.0,
            quality: 1.0,
        },
    );
    let before = tracker.get_reputation(&agent).unwrap().composite_score();
    tracker.decay_reputation(&agent, 0.5);
    let after = tracker.get_reputation(&agent).unwrap().composite_score();
    assert!(after < before);
}

#[test]
fn test_reputation_best_agent() {
    let mut tracker = reputation::ReputationTracker::new();
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    tracker.record_outcome(
        a1,
        &reputation::models::TaskOutcome {
            success: true,
            latency_ms: 50.0,
            cost: 1.0,
            quality: 0.99,
        },
    );
    tracker.record_outcome(
        a2,
        &reputation::models::TaskOutcome {
            success: false,
            latency_ms: 5000.0,
            cost: 100.0,
            quality: 0.1,
        },
    );
    let (best_id, _) = tracker.best_agent().unwrap();
    assert_eq!(best_id, a1);
}

// ═══════════════════════════════════════════════════════════════════
// Phase 8 — Consensus Tests (~35)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_consensus_round_creation() {
    let round = consensus::models::ConsensusRound::new(
        "choose framework",
        vec!["React".into(), "Vue".into()],
        vec![AgentId::new(), AgentId::new()],
    );
    assert_eq!(round.vote_count(), 0);
    assert!(!round.all_voted());
}

#[test]
fn test_consensus_vote_cast() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let mut round =
        consensus::models::ConsensusRound::new("test", vec!["A".into(), "B".into()], vec![a1, a2]);
    round.cast_vote(consensus::Vote::new(a1, "A", 0.9)).unwrap();
    assert_eq!(round.vote_count(), 1);
}

#[test]
fn test_consensus_duplicate_vote() {
    let a1 = AgentId::new();
    let mut round = consensus::models::ConsensusRound::new("test", vec!["A".into()], vec![a1]);
    round.cast_vote(consensus::Vote::new(a1, "A", 0.9)).unwrap();
    assert!(round.cast_vote(consensus::Vote::new(a1, "A", 0.9)).is_err());
}

#[test]
fn test_consensus_invalid_choice() {
    let a1 = AgentId::new();
    let mut round = consensus::models::ConsensusRound::new("test", vec!["A".into()], vec![a1]);
    assert!(round.cast_vote(consensus::Vote::new(a1, "B", 0.9)).is_err());
}

#[test]
fn test_consensus_non_participant() {
    let a1 = AgentId::new();
    let outsider = AgentId::new();
    let mut round = consensus::models::ConsensusRound::new("test", vec!["A".into()], vec![a1]);
    assert!(round
        .cast_vote(consensus::Vote::new(outsider, "A", 0.9))
        .is_err());
}

#[test]
fn test_consensus_majority_vote_agree() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let a3 = AgentId::new();
    let mut round = consensus::models::ConsensusRound::new(
        "test",
        vec!["A".into(), "B".into()],
        vec![a1, a2, a3],
    );
    round.cast_vote(consensus::Vote::new(a1, "A", 0.9)).unwrap();
    round.cast_vote(consensus::Vote::new(a2, "A", 0.8)).unwrap();
    round.cast_vote(consensus::Vote::new(a3, "B", 0.7)).unwrap();

    let result = consensus::ConsensusAlgorithm::MajorityVote.resolve(&round);
    assert!(result.is_agreed());
    assert_eq!(result, consensus::ConsensusResult::Agreed("A".into()));
}

#[test]
fn test_consensus_majority_vote_deadlock() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let mut round =
        consensus::models::ConsensusRound::new("test", vec!["A".into(), "B".into()], vec![a1, a2]);
    round.cast_vote(consensus::Vote::new(a1, "A", 0.9)).unwrap();
    round.cast_vote(consensus::Vote::new(a2, "B", 0.9)).unwrap();

    let result = consensus::ConsensusAlgorithm::MajorityVote.resolve(&round);
    assert!(result.is_deadlock());
}

#[test]
fn test_consensus_weighted_vote() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let mut round =
        consensus::models::ConsensusRound::new("test", vec!["A".into(), "B".into()], vec![a1, a2]);
    round
        .cast_vote(consensus::Vote::new(a1, "A", 0.9).with_weight(3.0))
        .unwrap();
    round
        .cast_vote(consensus::Vote::new(a2, "B", 0.9).with_weight(1.0))
        .unwrap();

    let result = consensus::ConsensusAlgorithm::WeightedVote.resolve(&round);
    assert_eq!(result, consensus::ConsensusResult::Agreed("A".into()));
}

#[test]
fn test_consensus_confidence_vote() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let a3 = AgentId::new();
    let mut round = consensus::models::ConsensusRound::new(
        "test",
        vec!["A".into(), "B".into()],
        vec![a1, a2, a3],
    );
    round
        .cast_vote(consensus::Vote::new(a1, "A", 0.95))
        .unwrap();
    round
        .cast_vote(consensus::Vote::new(a2, "A", 0.90))
        .unwrap();
    round
        .cast_vote(consensus::Vote::new(a3, "B", 0.30))
        .unwrap();

    let result = consensus::ConsensusAlgorithm::ConfidenceVote.resolve(&round);
    assert!(result.is_agreed());
}

#[test]
fn test_consensus_expert_override() {
    let expert = AgentId::new();
    let a2 = AgentId::new();
    let a3 = AgentId::new();
    let mut round = consensus::models::ConsensusRound::new(
        "test",
        vec!["A".into(), "B".into()],
        vec![expert, a2, a3],
    );
    round
        .cast_vote(consensus::Vote::new(expert, "A", 0.95).with_weight(3.0))
        .unwrap();
    round
        .cast_vote(consensus::Vote::new(a2, "B", 0.5).with_weight(1.0))
        .unwrap();
    round
        .cast_vote(consensus::Vote::new(a3, "B", 0.5).with_weight(1.0))
        .unwrap();

    let result = consensus::ConsensusAlgorithm::ExpertOverride {
        expert_confidence_threshold: 90,
    }
    .resolve(&round);
    assert_eq!(result, consensus::ConsensusResult::Agreed("A".into()));
}

#[test]
fn test_consensus_empty_votes() {
    let round =
        consensus::models::ConsensusRound::new("test", vec!["A".into()], vec![AgentId::new()]);
    let result = consensus::ConsensusAlgorithm::MajorityVote.resolve(&round);
    assert!(result.is_deadlock());
}

#[tokio::test]
async fn test_consensus_engine_full_flow() {
    let engine = consensus::ConsensusEngine::new();
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let a3 = AgentId::new();

    let round_id = engine
        .propose(
            "choose language",
            vec!["Rust".into(), "Go".into()],
            vec![a1, a2, a3],
            None,
        )
        .await
        .unwrap();

    engine
        .cast_vote(&round_id, consensus::Vote::new(a1, "Rust", 0.9))
        .await
        .unwrap();
    engine
        .cast_vote(&round_id, consensus::Vote::new(a2, "Rust", 0.8))
        .await
        .unwrap();
    engine
        .cast_vote(&round_id, consensus::Vote::new(a3, "Go", 0.7))
        .await
        .unwrap();

    let result = engine
        .resolve(&round_id, &consensus::ConsensusAlgorithm::MajorityVote)
        .await
        .unwrap();
    assert_eq!(result, consensus::ConsensusResult::Agreed("Rust".into()));
}

// ═══════════════════════════════════════════════════════════════════
// Phase 9 — Debate Tests (~30)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_debate_creation() {
    let d = debate::Debate::new("topic", AgentId::new(), AgentId::new(), AgentId::new());
    assert_eq!(d.state, debate::models::DebateState::AwaitingProposal);
    assert!(!d.is_resolved());
}

#[test]
fn test_debate_full_workflow() {
    let proposer = AgentId::new();
    let opponent = AgentId::new();
    let judge = AgentId::new();
    let mut d = debate::Debate::new("architecture", proposer, opponent, judge);

    d.submit_argument(debate::Argument::new(
        proposer,
        debate::DebateRole::Proposer,
        debate::models::ArgumentType::Proposal,
        "use microservices",
    ))
    .unwrap();
    d.submit_argument(debate::Argument::new(
        opponent,
        debate::DebateRole::Opponent,
        debate::models::ArgumentType::Counterargument,
        "monolith is simpler",
    ))
    .unwrap();
    d.submit_argument(debate::Argument::new(
        proposer,
        debate::DebateRole::Proposer,
        debate::models::ArgumentType::Rebuttal,
        "microservices scale better",
    ))
    .unwrap();
    d.submit_argument(debate::Argument::new(
        judge,
        debate::DebateRole::Judge,
        debate::models::ArgumentType::Verdict,
        "proposer wins",
    ))
    .unwrap();

    assert!(d.is_resolved());
    assert_eq!(d.argument_count(), 4);
}

#[test]
fn test_debate_wrong_author() {
    let proposer = AgentId::new();
    let opponent = AgentId::new();
    let judge = AgentId::new();
    let mut d = debate::Debate::new("test", proposer, opponent, judge);

    // Opponent trying to submit proposal
    let result = d.submit_argument(debate::Argument::new(
        opponent,
        debate::DebateRole::Opponent,
        debate::models::ArgumentType::Proposal,
        "wrong",
    ));
    assert!(result.is_err());
}

#[test]
fn test_debate_wrong_order() {
    let proposer = AgentId::new();
    let opponent = AgentId::new();
    let judge = AgentId::new();
    let mut d = debate::Debate::new("test", proposer, opponent, judge);

    // Counterargument before proposal
    let result = d.submit_argument(debate::Argument::new(
        opponent,
        debate::DebateRole::Opponent,
        debate::models::ArgumentType::Counterargument,
        "too early",
    ));
    assert!(result.is_err());
}

#[test]
fn test_debate_outcome() {
    assert!(debate::DebateOutcome::ProposerWins.is_decisive());
    assert!(debate::DebateOutcome::OpponentWins.is_decisive());
    assert!(!debate::DebateOutcome::Inconclusive.is_decisive());
}

#[tokio::test]
async fn test_debate_engine_flow() {
    let engine = debate::DebateEngine::new();
    let proposer = AgentId::new();
    let opponent = AgentId::new();
    let judge = AgentId::new();

    let id = engine
        .start_debate("architecture", proposer, opponent, judge, None)
        .await
        .unwrap();

    engine
        .submit_proposal(&id, proposer, "use Rust", 0.9)
        .await
        .unwrap();
    engine
        .submit_counterargument(&id, opponent, "use Go", 0.7)
        .await
        .unwrap();
    engine
        .submit_rebuttal(&id, proposer, "Rust is safer", 0.95)
        .await
        .unwrap();
    let outcome = engine
        .render_verdict(&id, judge, debate::DebateOutcome::ProposerWins, "Rust wins")
        .await
        .unwrap();

    assert_eq!(outcome, debate::DebateOutcome::ProposerWins);
    let transcript = engine.get_transcript(&id).await.unwrap();
    assert_eq!(transcript.len(), 4);
}

#[tokio::test]
async fn test_debate_engine_compromise() {
    let engine = debate::DebateEngine::new();
    let p = AgentId::new();
    let o = AgentId::new();
    let j = AgentId::new();

    let id = engine.start_debate("test", p, o, j, None).await.unwrap();
    engine
        .submit_proposal(&id, p, "option A", 0.6)
        .await
        .unwrap();
    engine
        .submit_counterargument(&id, o, "option B", 0.6)
        .await
        .unwrap();
    engine
        .submit_rebuttal(&id, p, "combine both", 0.7)
        .await
        .unwrap();
    let outcome = engine
        .render_verdict(
            &id,
            j,
            debate::DebateOutcome::Compromise("merged solution".into()),
            "both have merit",
        )
        .await
        .unwrap();

    assert!(!outcome.is_decisive());
}

// ═══════════════════════════════════════════════════════════════════
// Phase 10 — Verification Tests (~25)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_verification_create() {
    let engine = verification::VerificationEngine::new();
    let id = engine
        .create_verification(
            verification::VerificationPattern::CodeReview,
            "fn main() {}",
            AgentId::new(),
            AgentId::new(),
        )
        .await;
    assert_eq!(engine.verification_count().await, 1);
    let v = engine.get_verification(&id).await.unwrap();
    assert!(!v.is_resolved());
}

#[tokio::test]
async fn test_verification_approve() {
    let engine = verification::VerificationEngine::new();
    let id = engine
        .create_verification(
            verification::VerificationPattern::CodeReview,
            "code",
            AgentId::new(),
            AgentId::new(),
        )
        .await;
    engine
        .submit_review(
            &id,
            verification::VerificationResult::Approved { confidence: 0.95 },
            vec!["LGTM".into()],
        )
        .await
        .unwrap();
    let result = engine.get_result(&id).await.unwrap();
    assert!(result.is_approved());
}

#[tokio::test]
async fn test_verification_reject() {
    let engine = verification::VerificationEngine::new();
    let id = engine
        .create_verification(
            verification::VerificationPattern::PlanAudit,
            "plan",
            AgentId::new(),
            AgentId::new(),
        )
        .await;
    engine
        .submit_review(
            &id,
            verification::VerificationResult::Rejected {
                reason: "missing steps".into(),
            },
            vec![],
        )
        .await
        .unwrap();
    let result = engine.get_result(&id).await.unwrap();
    assert!(result.is_rejected());
}

#[tokio::test]
async fn test_verification_needs_revision() {
    let engine = verification::VerificationEngine::new();
    let id = engine
        .create_verification(
            verification::VerificationPattern::GenerateCritique,
            "output",
            AgentId::new(),
            AgentId::new(),
        )
        .await;
    engine
        .submit_review(
            &id,
            verification::VerificationResult::NeedsRevision {
                comments: vec!["add tests".into()],
            },
            vec!["add tests".into()],
        )
        .await
        .unwrap();
    let result = engine.get_result(&id).await.unwrap();
    assert!(!result.is_approved());
    assert!(!result.is_rejected());
}

#[tokio::test]
async fn test_verification_pending_for_verifier() {
    let engine = verification::VerificationEngine::new();
    let verifier = AgentId::new();
    engine
        .create_verification(
            verification::VerificationPattern::CodeReview,
            "code1",
            AgentId::new(),
            verifier,
        )
        .await;
    engine
        .create_verification(
            verification::VerificationPattern::CodeReview,
            "code2",
            AgentId::new(),
            verifier,
        )
        .await;

    let pending = engine.get_pending_for_verifier(&verifier).await;
    assert_eq!(pending.len(), 2);
}

#[test]
fn test_verification_patterns() {
    assert_ne!(
        verification::VerificationPattern::CodeReview,
        verification::VerificationPattern::PlanAudit
    );
    assert_ne!(
        verification::VerificationPattern::ReasonVerify,
        verification::VerificationPattern::GenerateCritique
    );
}

// ═══════════════════════════════════════════════════════════════════
// Phase 11 — Conflict Tests (~25)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_conflict_creation() {
    let c = conflict::Conflict::new(
        conflict::ConflictType::ResourceContention,
        vec![AgentId::new(), AgentId::new()],
        "both need GPU",
    );
    assert!(!c.is_resolved());
    assert_eq!(c.conflict_type, conflict::ConflictType::ResourceContention);
}

#[test]
fn test_conflict_resolve() {
    let mut c = conflict::Conflict::new(
        conflict::ConflictType::ContradictoryPlans,
        vec![AgentId::new()],
        "test",
    );
    let winner = AgentId::new();
    c.resolve(conflict::Resolution::PriorityBased(winner));
    assert!(c.is_resolved());
}

#[test]
fn test_conflict_unresolvable() {
    let mut c = conflict::Conflict::new(
        conflict::ConflictType::PriorityClash,
        vec![AgentId::new()],
        "test",
    );
    c.mark_unresolvable("deadlocked");
    assert!(c.is_resolved());
}

#[test]
fn test_conflict_classify() {
    assert_eq!(
        conflict::engine::ConflictResolver::classify_conflict("resource contention"),
        conflict::ConflictType::ResourceContention
    );
    assert_eq!(
        conflict::engine::ConflictResolver::classify_conflict("contradictory plans"),
        conflict::ConflictType::ContradictoryPlans
    );
    assert_eq!(
        conflict::engine::ConflictResolver::classify_conflict("duplicate work detected"),
        conflict::ConflictType::DuplicateWork
    );
    assert_eq!(
        conflict::engine::ConflictResolver::classify_conflict("priority issue"),
        conflict::ConflictType::PriorityClash
    );
}

#[test]
fn test_conflict_resolver_register_and_resolve() {
    let mut resolver = conflict::ConflictResolver::new();
    let c = conflict::Conflict::new(
        conflict::ConflictType::DuplicateWork,
        vec![AgentId::new(), AgentId::new()],
        "both doing same task",
    );
    let id = resolver.register_conflict(c);
    assert_eq!(resolver.conflict_count(), 1);
    assert_eq!(resolver.get_unresolved().len(), 1);

    resolver
        .resolve(&id, conflict::Resolution::Escalate)
        .unwrap();
    assert_eq!(resolver.get_unresolved().len(), 0);
    assert_eq!(resolver.resolved_count(), 1);
}

#[test]
fn test_conflict_detect_contradictory() {
    let mut resolver = conflict::ConflictResolver::new();
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let facts = vec![
        shared_memory::SharedFact::new(a1, shared_memory::FactCategory::Fact, "framework", "React"),
        shared_memory::SharedFact::new(a2, shared_memory::FactCategory::Fact, "framework", "Vue"),
    ];
    let conflicts = resolver.detect_conflicts(&facts);
    assert!(!conflicts.is_empty());
}

#[test]
fn test_conflict_detect_duplicate_work() {
    let mut resolver = conflict::ConflictResolver::new();
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let facts = vec![
        shared_memory::SharedFact::new(
            a1,
            shared_memory::FactCategory::Fact,
            "task:implement_auth",
            "working",
        ),
        shared_memory::SharedFact::new(
            a2,
            shared_memory::FactCategory::Fact,
            "task:implement_auth",
            "working",
        ),
    ];
    let conflicts = resolver.detect_conflicts(&facts);
    assert!(!conflicts.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// Phase 12 — Swarm Tests (~30)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_swarm_strategy_min_agents() {
    assert_eq!(swarm::SwarmStrategy::Parallel.recommended_min_agents(), 2);
    assert_eq!(
        swarm::SwarmStrategy::Hierarchical.recommended_min_agents(),
        3
    );
    assert_eq!(swarm::SwarmStrategy::Adaptive.recommended_min_agents(), 3);
}

#[test]
fn test_swarm_result_from_results() {
    let results = vec![
        swarm::models::SwarmAgentResult {
            agent_id: AgentId::new(),
            output: "a".into(),
            quality_score: 0.9,
            latency_ms: 100,
            success: true,
        },
        swarm::models::SwarmAgentResult {
            agent_id: AgentId::new(),
            output: "b".into(),
            quality_score: 0.7,
            latency_ms: 200,
            success: true,
        },
        swarm::models::SwarmAgentResult {
            agent_id: AgentId::new(),
            output: "c".into(),
            quality_score: 0.3,
            latency_ms: 50,
            success: false,
        },
    ];
    let result = swarm::models::SwarmResult::from_results(results);
    assert!((result.success_rate - 2.0 / 3.0).abs() < 0.01);
    assert!(result.best_result.is_some());
    assert!((result.best_result.unwrap().quality_score - 0.9).abs() < f64::EPSILON);
}

#[test]
fn test_swarm_execution_creation() {
    let agents = vec![AgentId::new(), AgentId::new()];
    let exec = swarm::models::SwarmExecution::new("test task", agents.clone());
    assert_eq!(exec.agent_count(), 2);
    assert_eq!(exec.result_count(), 0);
}

#[test]
fn test_swarm_execution_finalize() {
    let agents = vec![AgentId::new(), AgentId::new()];
    let mut exec = swarm::models::SwarmExecution::new("test", agents.clone());
    exec.add_result(swarm::models::SwarmAgentResult {
        agent_id: agents[0],
        output: "result_a".into(),
        quality_score: 0.8,
        latency_ms: 100,
        success: true,
    });
    exec.add_result(swarm::models::SwarmAgentResult {
        agent_id: agents[1],
        output: "result_b".into(),
        quality_score: 0.6,
        latency_ms: 150,
        success: true,
    });

    let result = exec.finalize();
    assert_eq!(exec.state, swarm::SwarmState::Completed);
    assert!(result.best_result.is_some());
}

#[tokio::test]
async fn test_swarm_engine_launch() {
    let engine = swarm::SwarmEngine::new();
    let agents = vec![AgentId::new(), AgentId::new()];
    let id = engine
        .launch_swarm(&swarm::SwarmStrategy::Parallel, "test", agents, None)
        .await
        .unwrap();
    assert_eq!(engine.swarm_count().await, 1);
    let exec = engine.get_execution(&id).await.unwrap();
    assert_eq!(exec.state, swarm::SwarmState::Running);
}

#[tokio::test]
async fn test_swarm_engine_insufficient_agents() {
    let engine = swarm::SwarmEngine::new();
    let agents = vec![AgentId::new()]; // Need at least 2 for Parallel
    let result = engine
        .launch_swarm(&swarm::SwarmStrategy::Parallel, "test", agents, None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_swarm_engine_governor_deny() {
    let rules = governor::GovernorRules {
        max_swarm_size: 2,
        ..Default::default()
    };
    let gov = governor::SafetyGovernor::new(rules);
    let engine = swarm::SwarmEngine::new();
    let agents = vec![AgentId::new(), AgentId::new(), AgentId::new()];
    let result = engine
        .launch_swarm(&swarm::SwarmStrategy::Parallel, "test", agents, Some(&gov))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_swarm_engine_submit_and_collect() {
    let engine = swarm::SwarmEngine::new();
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let id = engine
        .launch_swarm(&swarm::SwarmStrategy::Parallel, "test", vec![a1, a2], None)
        .await
        .unwrap();

    engine
        .submit_result(
            &id,
            swarm::models::SwarmAgentResult {
                agent_id: a1,
                output: "a".into(),
                quality_score: 0.9,
                latency_ms: 100,
                success: true,
            },
        )
        .await
        .unwrap();
    engine
        .submit_result(
            &id,
            swarm::models::SwarmAgentResult {
                agent_id: a2,
                output: "b".into(),
                quality_score: 0.7,
                latency_ms: 200,
                success: true,
            },
        )
        .await
        .unwrap();

    let result = engine.collect_results(&id).await.unwrap();
    assert!(result.best_result.is_some());
    assert_eq!(result.best_result.unwrap().agent_id, a1);
}

#[tokio::test]
async fn test_swarm_engine_terminate() {
    let engine = swarm::SwarmEngine::new();
    let agents = vec![AgentId::new(), AgentId::new()];
    let id = engine
        .launch_swarm(&swarm::SwarmStrategy::Parallel, "test", agents, None)
        .await
        .unwrap();
    engine.terminate_swarm(&id).await.unwrap();
    let exec = engine.get_execution(&id).await.unwrap();
    assert_eq!(exec.state, swarm::SwarmState::Terminated);
}

#[tokio::test]
async fn test_swarm_non_member_submit() {
    let engine = swarm::SwarmEngine::new();
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let outsider = AgentId::new();
    let id = engine
        .launch_swarm(&swarm::SwarmStrategy::Parallel, "test", vec![a1, a2], None)
        .await
        .unwrap();

    let result = engine
        .submit_result(
            &id,
            swarm::models::SwarmAgentResult {
                agent_id: outsider,
                output: "bad".into(),
                quality_score: 0.5,
                latency_ms: 100,
                success: true,
            },
        )
        .await;
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// Phase 13-14 — Knowledge Sync & Org Learning Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_org_learning_team() {
    let mut engine = organizational_learning::OrgLearningEngine::new();
    engine.record_team_outcome("eng_team", true, 0.9, 10.0);
    engine.record_team_outcome("eng_team", true, 0.8, 15.0);
    engine.record_team_outcome("eng_team", false, 0.3, 50.0);

    let perf = engine.get_team("eng_team").unwrap();
    assert_eq!(perf.sample_count, 3);
}

#[test]
fn test_org_learning_pair() {
    let mut engine = organizational_learning::OrgLearningEngine::new();
    engine.record_pair_outcome("coder+reviewer", 0.9, 0.85);
    engine.record_pair_outcome("coder+reviewer", 0.95, 0.9);
    let perf = engine.get_pair("coder+reviewer").unwrap();
    assert_eq!(perf.sample_count, 2);
}

#[test]
fn test_org_learning_workflow() {
    let mut engine = organizational_learning::OrgLearningEngine::new();
    engine.record_workflow_outcome("pipeline_v1", true, 5.0);
    engine.record_workflow_outcome("pipeline_v1", true, 6.0);
    let perf = engine.get_workflow("pipeline_v1").unwrap();
    assert_eq!(perf.sample_count, 2);
}

#[test]
fn test_org_learning_recommend_team() {
    let mut engine = organizational_learning::OrgLearningEngine::new();
    for _ in 0..5 {
        engine.record_team_outcome("good_team", true, 0.9, 5.0);
        engine.record_team_outcome("bad_team", false, 0.2, 50.0);
    }
    let rec = engine.recommend_team().unwrap();
    assert_eq!(rec.team_key, "good_team");
}

#[test]
fn test_org_learning_recommend_pair() {
    let mut engine = organizational_learning::OrgLearningEngine::new();
    for _ in 0..3 {
        engine.record_pair_outcome("good_pair", 0.95, 0.9);
        engine.record_pair_outcome("bad_pair", 0.2, 0.1);
    }
    let rec = engine.recommend_pair().unwrap();
    assert_eq!(rec.pair_key, "good_pair");
}

#[test]
fn test_org_learning_total_tracked() {
    let mut engine = organizational_learning::OrgLearningEngine::new();
    engine.record_team_outcome("t1", true, 0.9, 5.0);
    engine.record_pair_outcome("p1", 0.9, 0.9);
    engine.record_workflow_outcome("w1", true, 5.0);
    assert_eq!(engine.total_tracked(), 3);
}

// ═══════════════════════════════════════════════════════════════════
// Phase 15 — Distributed Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_worker_node_local() {
    let node = distributed::WorkerNode::local(16);
    assert!(node.is_available());
    assert_eq!(node.available_slots(), 16);
}

#[test]
fn test_worker_node_allocation() {
    let mut node = distributed::WorkerNode::local(2);
    node.allocate_agent().unwrap();
    node.allocate_agent().unwrap();
    assert!(!node.is_available());
    assert!(node.allocate_agent().is_err());
    node.release_agent();
    assert!(node.is_available());
}

#[test]
fn test_cluster_single_node() {
    let cluster = distributed::ClusterMembership::single_node(16);
    assert!(cluster.is_single_node());
    assert_eq!(cluster.node_count(), 1);
    assert!(cluster.leader().is_some());
    assert_eq!(cluster.total_capacity(), 16);
}

#[test]
fn test_cluster_add_remove_node() {
    let mut cluster = distributed::ClusterMembership::new();
    let node = distributed::WorkerNode::local(8);
    let id = node.id;
    cluster.add_node(node);
    assert_eq!(cluster.node_count(), 1);
    cluster.remove_node(&id);
    assert_eq!(cluster.node_count(), 0);
}

#[test]
fn test_leader_election() {
    let cluster = distributed::ClusterMembership::single_node(16);
    let leader = distributed::LeaderElection::elect_leader(&cluster);
    assert!(leader.is_some());
    assert!(distributed::LeaderElection::is_leader(
        &cluster,
        &leader.unwrap()
    ));
}

#[test]
fn test_heartbeat_check() {
    let mut cluster = distributed::ClusterMembership::single_node(16);
    let heartbeat = distributed::Heartbeat::new(30);
    let stale = heartbeat.check_health(&mut cluster);
    assert!(stale.is_empty()); // Fresh node, not stale
}

// ═══════════════════════════════════════════════════════════════════
// Phase 16 — Telemetry Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_coordination_metrics() {
    let metrics = telemetry::CoordinationMetrics::new();
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    // Smoke tests — just verify they don't panic
    metrics.record_message_sent(&a1, Some(&a2), "direct");
    metrics.record_delegation("task_1", &a1, &a2);
    metrics.record_consensus_decision("framework", "majority", true);
    metrics.record_debate("architecture", "proposer_wins");
    metrics.record_conflict("resource_contention", true);
    metrics.record_swarm_execution("parallel", 5, 0.8);
    metrics.record_verification("code_review", true);
    metrics.record_reputation_update(&a1, 0.85);
    metrics.record_governor_decision("delegation", true);
    metrics.record_resource_utilization(0.5, 0.3, 0.2);
}

// ═══════════════════════════════════════════════════════════════════
// Phase 17 — Integration Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_coordination_pipeline_creation() {
    let pipeline = coordination::CoordinationPipeline::new("build auth system");
    assert_eq!(pipeline.stages_completed(), 1);
    assert!(!pipeline.is_complete());
}

#[test]
fn test_coordination_pipeline_advance() {
    let mut pipeline = coordination::CoordinationPipeline::new("test");
    pipeline.advance(coordination::pipeline::PipelineStage::OrganizationBuilt);
    pipeline.advance(coordination::pipeline::PipelineStage::ResourcesChecked);
    assert_eq!(pipeline.stages_completed(), 3);
}

#[test]
fn test_coordination_pipeline_complete() {
    let mut pipeline = coordination::CoordinationPipeline::new("test");
    pipeline.complete();
    assert!(pipeline.is_complete());
    assert!(pipeline.duration_secs().is_some());
}

#[test]
fn test_coordination_pipeline_fail() {
    let mut pipeline = coordination::CoordinationPipeline::new("test");
    pipeline.fail("resource exhaustion");
    assert!(pipeline.is_complete());
}

#[test]
fn test_agent_coordinator_creation() {
    let coordinator = coordination::AgentCoordinator::default();
    assert!(!coordinator.is_ready());
}

#[test]
fn test_agent_coordinator_build_org() {
    let mut coordinator = coordination::AgentCoordinator::default();
    coordinator.build_organization("implement feature");
    assert!(coordinator.is_ready());
    let org = coordinator.get_organization().unwrap();
    assert!(org.node_count() >= 7); // CEO + planning(3) + engineering(3)
}

// ═══════════════════════════════════════════════════════════════════
// Determinism Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_deterministic_consensus() {
    // Same inputs should produce same results
    for _ in 0..5 {
        let a1 = AgentId(uuid::Uuid::from_u128(1));
        let a2 = AgentId(uuid::Uuid::from_u128(2));
        let a3 = AgentId(uuid::Uuid::from_u128(3));
        let mut round = consensus::models::ConsensusRound::new(
            "test",
            vec!["A".into(), "B".into()],
            vec![a1, a2, a3],
        );
        round.cast_vote(consensus::Vote::new(a1, "A", 0.9)).unwrap();
        round.cast_vote(consensus::Vote::new(a2, "A", 0.8)).unwrap();
        round.cast_vote(consensus::Vote::new(a3, "B", 0.7)).unwrap();
        let result = consensus::ConsensusAlgorithm::MajorityVote.resolve(&round);
        assert_eq!(result, consensus::ConsensusResult::Agreed("A".into()));
    }
}

#[test]
fn test_deterministic_reputation() {
    for _ in 0..5 {
        let mut tracker = reputation::ReputationTracker::new();
        let agent = AgentId(uuid::Uuid::from_u128(42));
        tracker.record_outcome(
            agent,
            &reputation::models::TaskOutcome {
                success: true,
                latency_ms: 100.0,
                cost: 5.0,
                quality: 0.9,
            },
        );
        tracker.record_outcome(
            agent,
            &reputation::models::TaskOutcome {
                success: false,
                latency_ms: 500.0,
                cost: 20.0,
                quality: 0.3,
            },
        );
        let rep = tracker.get_reputation(&agent).unwrap();
        // EMA should be deterministic
        assert!((rep.success_rate - 0.8).abs() < 0.01);
    }
}

#[test]
fn test_deterministic_task_splitting() {
    let splitter = delegation::TaskSplitter::new();
    for _ in 0..10 {
        let result = splitter.analyze("build API", 0.75);
        assert_eq!(result.strategy, delegation::SplitStrategy::Parallel);
        assert_eq!(result.sub_tasks.len(), 2);
    }
}

#[test]
fn test_deterministic_org_learning() {
    for _ in 0..5 {
        let mut engine = organizational_learning::OrgLearningEngine::new();
        engine.record_team_outcome("team_a", true, 0.9, 5.0);
        engine.record_team_outcome("team_a", false, 0.3, 50.0);
        let perf = engine.get_team("team_a").unwrap();
        // EMA with alpha=0.2: 0.2*0 + 0.8*0.9 = 0.72 (for success_rate after failure)
        assert!((perf.ema_success_rate - 0.2 * 0.0 - 0.8 * 1.0).abs() < 0.01);
    }
}
