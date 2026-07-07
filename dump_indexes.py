import sqlite3
db = sqlite3.connect('E:\\My Projects\\vscode\\.ares\\ares.db')
cur = db.cursor()
indexes = cur.execute("SELECT name, tbl_name, sql FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY tbl_name, name").fetchall()
print(f'Total: {len(indexes)}')
for name, tbl, sql in indexes:
    sql_short = (sql or 'auto-index')[:80]
    print(f'{tbl:25s} {name:45s} {sql_short}')
