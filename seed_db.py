import sqlite3

conn = sqlite3.connect('memory.db')
cursor = conn.cursor()

project_id = 'PROJ-001'
edges = [
    ('edge-1', project_id, 'req-auth', 'dec-auth', 'motivated_by', 1.0, 1.0, 'agent', 0, None, 0),
    ('edge-2', project_id, 'dec-auth', 'arch-auth', 'derived_from', 1.0, 1.0, 'agent', 0, None, 0),
    ('edge-3', project_id, 'arch-auth', 'crates/auth/src/lib.rs', 'implements', 1.0, 1.0, 'agent', 0, None, 0),
    ('edge-4', project_id, 'arch-auth', 'crates/auth', 'implements', 1.0, 1.0, 'agent', 0, None, 0),
    ('edge-5', project_id, 'crates/auth', 'test-auth', 'validated_by', 1.0, 1.0, 'agent', 0, None, 0),
    ('edge-6', project_id, 'crates/auth', 'test-session', 'validated_by', 1.0, 1.0, 'agent', 0, None, 0),
    ('edge-7', project_id, 'test-auth', 'runtime-auth', 'temporal_follows', 1.0, 1.0, 'agent', 0, None, 0),
]

try:
    cursor.executemany("INSERT INTO graph_edges (id, project_id, from_node_id, to_node_id, edge_type, weight, confidence, source, valid_from, valid_until, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", edges)
    print("Edges inserted successfully.")
except Exception as e:
    print(f"Error: {e}")

conn.commit()
conn.close()
