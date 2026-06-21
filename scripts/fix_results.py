import sqlite3
import json
import os

results_path = "reports/validation/real_world_results.json"
with open(results_path, 'r') as f:
    results = json.load(f)

for res in results:
    if res['name'] == 'ARES':
        db_path = ".ares/ares.db"
    elif res['name'] == 'Automyra':
        db_path = ".temp/automyra/.ares/ares.db"
    elif res['name'] == 'ripgrep':
        db_path = ".temp/ripgrep/.ares/ares.db"
    elif res['name'] == 'cargo-watch':
        db_path = ".temp/cargo-watch/.ares/ares.db"
    elif res['name'] == 'Next.js':
        db_path = ".temp/nextjs/.ares/ares.db"
    elif res['name'] == 'NestJS':
        db_path = ".temp/nestjs/.ares/ares.db"
    elif res['name'] == 'Turborepo':
        db_path = ".temp/turborepo/.ares/ares.db"
    elif res['name'] == 'Nx Workspace':
        db_path = ".temp/nx/.ares/ares.db"
        
    if os.path.exists(db_path):
        conn = sqlite3.connect(db_path)
        c = conn.cursor()
        
        try:
            c.execute("SELECT COUNT(*) FROM graph_entities")
            node_count = c.fetchone()[0]
            
            c.execute("SELECT COUNT(*) FROM graph_relationships")
            edge_count = c.fetchone()[0]
            
            c.execute("SELECT COUNT(*) FROM graph_entities WHERE entity_type = 'KnowledgeGap'")
            gap_count = c.fetchone()[0]
            
            c.execute("SELECT COUNT(*) FROM graph_entities WHERE entity_type = 'CodeArtifact'")
            code_nodes = c.fetchone()[0]
            
            c.execute("SELECT COUNT(DISTINCT source_entity) FROM graph_relationships WHERE relationship_type = 'ValidatedBy'")
            traced = c.fetchone()[0]
            
            trace_score = 0.0
            if code_nodes > 0:
                trace_score = (traced / code_nodes) * 100.0
                
            res['node_count'] = node_count
            res['edge_count'] = edge_count
            res['gap_count'] = gap_count
            res['traceability_score'] = trace_score
        except Exception as e:
            print(f"Error on {res['name']}: {e}")
        
        conn.close()
    else:
        print(f"DB not found: {db_path}")

with open(results_path, 'w') as f:
    json.dump(results, f, indent=2)

print("Updated results.")
