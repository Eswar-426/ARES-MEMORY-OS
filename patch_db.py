import sqlite3

conn = sqlite3.connect('memory.db')
cursor = conn.cursor()

# Get existing nodes
nodes = cursor.execute("SELECT id, project_id, node_type, label, properties, file_path, created_at, updated_at, deleted_at FROM graph_nodes").fetchall()

# Drop table
cursor.execute("DROP TABLE graph_nodes")

# Recreate table with new constraints
cursor.execute("""
CREATE TABLE "graph_nodes" (
  id            TEXT PRIMARY KEY,
  project_id    TEXT NOT NULL REFERENCES projects(id),
  node_type     TEXT NOT NULL
                CHECK(node_type IN (
                  'project','file','function','method','class','struct','enum','trait','interface',
                  'module','service','decision','feature',
                  'bug','concept','tag','requirement','architecture','evidence','owner','repository',
                  'alternative','assumption','risk','folder','person',
                  'commit','branch','release','test','runtime_signal','outcome'
                )),
  label         TEXT NOT NULL,
  properties    TEXT NOT NULL,
  file_path     TEXT,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  deleted_at    INTEGER
)
""")

# Reinsert nodes
cursor.executemany("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, file_path, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", nodes)

conn.commit()

# Now reinsert the missing test-auth and test-session nodes
project_id = 'PROJ-001'
new_nodes = [
    ('test-auth', project_id, 'test', 'auth_login_test', '{}', None, 0, 0, None),
    ('test-session', project_id, 'test', 'session_expiry_test', '{}', None, 0, 0, None),
    ('runtime-auth', project_id, 'runtime_signal', 'Authentication Pipeline', '{}', None, 0, 0, None),
]
try:
    cursor.executemany("INSERT OR IGNORE INTO graph_nodes (id, project_id, node_type, label, properties, file_path, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", new_nodes)
    print("New nodes inserted.")
except Exception as e:
    print(e)

conn.commit()
conn.close()
