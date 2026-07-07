import sqlite3

output_file = r'C:\Users\eswar\.gemini\antigravity-ide\brain\e99bab2d-b695-4fe2-a9e0-6579d3f6bd9f\index_list.md'
db = sqlite3.connect(r'E:\My Projects\vscode\.ares\ares.db')
cur = db.cursor()
indexes = cur.execute("SELECT name, tbl_name, sql FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY tbl_name, name").fetchall()

with open(output_file, 'w', encoding='utf-8') as f:
    f.write(f'# Database Index List\n\n')
    f.write(f'Total non-system indexes: {len(indexes)}\n\n')
    f.write('| Table | Index Name | SQL |\n')
    f.write('|-------|------------|-----|\n')
    for name, tbl, sql in indexes:
        sql_clean = (sql or 'auto-index').replace('\n', ' ')
        f.write(f'| `{tbl}` | `{name}` | `{sql_clean}` |\n')
