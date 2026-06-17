import sqlite3
import urllib.request
import urllib.error
import json
import subprocess
import time
import uuid

DB_PATH = '.ares/ares.db'
API_BASE = 'http://127.0.0.1:8080/api/v1'

def seed_db():
    conn = sqlite3.connect(DB_PATH)
    cur = conn.cursor()
    
    # 1. Ensure project exists
    project_id = 'PRJ-DEMO'
    cur.execute("INSERT OR IGNORE INTO projects (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)", 
                (project_id, 'Demo Project', int(time.time()), int(time.time())))
    
    # Nodes to insert
    nodes = [
        ('REQ-AUTH', 'feature', 'Authentication Requirement'),
        ('DEC-JWT', 'decision', 'Use JWT for Auth'),
        ('ARCH-AUTH', 'concept', 'Auth Architecture'),
        ('CODE-auth.rs', 'file', 'auth.rs code'),
        ('TEST-auth_test.rs', 'file', 'auth.rs tests'),
        ('RUNTIME-auth_success', 'concept', 'Auth Success Runtime'),
        ('OUTCOME-user_login_success', 'concept', 'User Login Success')
    ]
    
    now = int(time.time())
    for (nid, ntype, lbl) in nodes:
        cur.execute("""
            INSERT OR IGNORE INTO graph_nodes 
            (id, project_id, node_type, label, properties, created_at, updated_at) 
            VALUES (?, ?, ?, ?, '{}', ?, ?)
        """, (nid, project_id, ntype, lbl, now, now))
    
    # Edges to insert
    edges = [
        ('REQ-AUTH', 'DEC-JWT', 'motivated_by'),
        ('DEC-JWT', 'ARCH-AUTH', 'motivated_by'),
        ('ARCH-AUTH', 'CODE-auth.rs', 'implements'),
        ('CODE-auth.rs', 'TEST-auth_test.rs', 'related_to'),
        ('TEST-auth_test.rs', 'RUNTIME-auth_success', 'related_to'),
        ('RUNTIME-auth_success', 'OUTCOME-user_login_success', 'caused')
    ]
    
    for (source, target, etype) in edges:
        eid = str(uuid.uuid4())
        cur.execute("""
            INSERT OR IGNORE INTO graph_edges 
            (id, project_id, from_node_id, to_node_id, edge_type, valid_from, created_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """, (eid, project_id, source, target, etype, now, now))

    # Add a memory revision for evolution
    rev_id = str(uuid.uuid4())
    cur.execute("""
        INSERT OR IGNORE INTO memory_revisions
        (revision_id, entity_id, entity_type, change_type, changed_at, changed_by, reason)
        VALUES (?, ?, ?, ?, ?, ?, ?)
    """, (rev_id, 'REQ-AUTH', 'feature', 'Created', now, 'System', 'Initial trace creation'))

    conn.commit()
    print("Database seeded with verification trace.")

def verify_api():
    endpoints = [
        f"{API_BASE}/memory/certification",
        f"{API_BASE}/memory/why/REQ-AUTH",
        f"{API_BASE}/memory/who/REQ-AUTH",
        f"{API_BASE}/memory/evolution/REQ-AUTH",
        f"{API_BASE}/memory/impact/REQ-AUTH",
    ]
    
    print("\n--- API End-to-End Verification ---")
    for url in endpoints:
        req = urllib.request.Request(url)
        try:
            with urllib.request.urlopen(req) as response:
                data = json.loads(response.read().decode())
                status = data.get("status")
                req_id = data.get("request_id")
                print(f"OK: {url.split('/')[-2]}/{url.split('/')[-1]} -> status: {status}, request_id: {req_id}")
        except urllib.error.URLError as e:
            print(f"FAILED: {url} -> {e}")

def verify_mcp():
    print("\n--- MCP Tool Verification ---")
    try:
        proc = subprocess.Popen(
            ['cargo', 'run', '-q', '-p', 'ares-mcp'],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True
        )
        
        # Initialize
        init_req = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}
        }
        proc.stdin.write(json.dumps(init_req) + "\n")
        proc.stdin.flush()
        
        init_resp = json.loads(proc.stdout.readline())
        print(f"MCP Initialize: {init_resp.get('id')} OK")
        
        # Tools call
        tool_req = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "ares_why_exists",
                "arguments": {"id": "REQ-AUTH"}
            }
        }
        proc.stdin.write(json.dumps(tool_req) + "\n")
        proc.stdin.flush()
        
        tool_resp = json.loads(proc.stdout.readline())
        content = tool_resp.get("result", {}).get("content", [])
        if content:
            print(f"MCP Tool Call ares_why_exists: Received Content Successfully")
        else:
            print(f"MCP Tool Call ares_why_exists: Failed or empty content: {tool_resp}")
        
        proc.terminate()
    except Exception as e:
        print(f"MCP Verification Failed: {e}")

if __name__ == '__main__':
    seed_db()
    verify_api()
    verify_mcp()
