import sqlite3

conn = sqlite3.connect('.ares/ares.db')
c = conn.cursor()

def fetch_count(query, params=()):
    c.execute(query, params)
    return c.fetchone()[0]

total_req = fetch_count('SELECT COUNT(*) FROM graph_entities WHERE entity_type = \'Requirement\'')
req_with_code = fetch_count('''
    SELECT COUNT(DISTINCT r.source_entity)
    FROM graph_relationships r
    JOIN graph_entities e ON r.source_entity = e.id
    WHERE e.entity_type = 'Requirement' AND r.relationship_type = 'ImplementedBy'
''')

req_with_tests = fetch_count('''
    SELECT COUNT(DISTINCT e.id)
    FROM graph_entities e
    JOIN graph_relationships r1 ON e.id = r1.source_entity AND r1.relationship_type = 'ImplementedBy'
    JOIN graph_relationships r2 ON r1.target_entity = r2.source_entity AND r2.relationship_type = 'ValidatedBy'
    WHERE e.entity_type = 'Requirement'
''')

total_dec = fetch_count('SELECT COUNT(*) FROM graph_entities WHERE entity_type = \'Decision\'')
dec_with_code = fetch_count('''
    SELECT COUNT(DISTINCT r.source_entity)
    FROM graph_relationships r
    JOIN graph_entities e ON r.source_entity = e.id
    WHERE e.entity_type = 'Decision' AND r.relationship_type = 'Drives'
''')

dec_with_evidence = fetch_count('''
    SELECT COUNT(DISTINCT r.target_entity)
    FROM graph_relationships r
    JOIN graph_entities e ON r.target_entity = e.id
    WHERE e.entity_type = 'Decision' AND r.relationship_type = 'Supports'
''')

total_files = fetch_count('SELECT COUNT(*) FROM graph_entities WHERE entity_type = \'CodeArtifact\'')

files_without_rationale = fetch_count('''
    SELECT COUNT(*) FROM graph_entities e
    WHERE e.entity_type = 'CodeArtifact' AND NOT EXISTS (
        SELECT 1 FROM graph_relationships r 
        WHERE r.target_entity = e.id AND r.relationship_type IN ('Drives', 'ImplementedBy')
    )
''')

files_without_owner = fetch_count('''
    SELECT COUNT(*) FROM graph_entities e
    WHERE e.entity_type = 'CodeArtifact' AND NOT EXISTS (
        SELECT 1 FROM graph_relationships r 
        WHERE r.source_entity = e.id AND r.relationship_type = 'OwnedBy'
    )
''')

files_without_tests = fetch_count('''
    SELECT COUNT(*) FROM graph_entities e
    WHERE e.entity_type = 'CodeArtifact' AND e.id NOT LIKE '%test%' AND NOT EXISTS (
        SELECT 1 FROM graph_relationships r 
        WHERE r.source_entity = e.id AND r.relationship_type = 'ValidatedBy'
    )
''')

req_code_pct = (req_with_code / max(1, total_req)) * 100
req_test_pct = (req_with_tests / max(1, total_req)) * 100
dec_code_pct = (dec_with_code / max(1, total_dec)) * 100
dec_evi_pct = (dec_with_evidence / max(1, total_dec)) * 100

md = f"""# ARES Self-Hosting Readiness Report

## Methodology Validation
This report analyzes how strictly ARES adheres to its own Memory OS framework.

| Metric | Result | Target |
|--------|--------|--------|
| **Total Requirements** | {total_req} | N/A |
| **Requirements with Code** | {req_code_pct:.1f}% | 100% |
| **Requirements with Tests** | {req_test_pct:.1f}% | 100% |
| **Total Decisions** | {total_dec} | N/A |
| **Decisions linked to Code** | {dec_code_pct:.1f}% | 100% |
| **Decisions linked to Evidence** | {dec_evi_pct:.1f}% | 100% |

## Repository Hygiene
| Metric | Count | Context (Total Files: {total_files}) |
|--------|-------|---------------------------------------|
| **Files without Rationale** | {files_without_rationale} | Files not mapped to a Requirement or Decision |
| **Files without Owner** | {files_without_owner} | Files not mapped in CODEOWNERS or equivalent |
| **Files without Tests** | {files_without_tests} | Implementation code lacking a ValidatedBy edge |

## Conclusion
ARES is generating these metrics natively from its own internal SQLite Knowledge Graph.
"""

with open('reports/validation/self_hosting_readiness.md', 'w', encoding='utf-8') as f:
    f.write(md)

print('Report generated successfully.')
