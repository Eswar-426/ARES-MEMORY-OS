import sqlite3
db = sqlite3.connect('E:\\My Projects\\vscode\\.ares\\ares.db')
cur = db.cursor()
indexes = cur.execute("SELECT name, tbl_name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'").fetchall()
print(f'Total non-system indexes: {len(indexes)}')
for name, tbl in sorted(indexes):
    print(f'  {tbl:25s} -> {name}')
