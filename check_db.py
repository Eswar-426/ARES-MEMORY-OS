import sqlite3
import sys
db = sqlite3.connect('E:\\My Projects\\vscode\\.ares\\ares.db')
cur = db.cursor()

# Get all tables including sqlite internals (FTS shadow tables)
tables = [r[0] for r in cur.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()]

print("--- Table Sizes ---")
total_size = 0
for t in tables:
    count = cur.execute(f'SELECT COUNT(*) FROM [{t}]').fetchone()[0]
    cols = [c[1] for c in cur.execute(f'PRAGMA table_info([{t}])').fetchall()]
    if cols:
        size_query = 'SELECT SUM(' + '+'.join(f'LENGTH(COALESCE(CAST([{c}] AS BLOB), ""))' for c in cols) + ') FROM [' + t + ']'
        try:
            size = cur.execute(size_query).fetchone()[0] or 0
        except Exception as e:
            # print(f'Error calculating size for {t}: {e}')
            size = 0
    else:
        size = 0
    total_size += size
    mb = size / (1024*1024)
    if mb > 0 or count > 0:
        print(f'{t:40s} {count:>8,} rows  {mb:>10.1f} MB')

print(f'\nTotal estimated raw data size: {total_size/(1024*1024):.1f} MB')

# Let's count page counts using sqlite3 analyzer logic:
# `PRAGMA page_count` and `PRAGMA page_size`
page_count = cur.execute("PRAGMA page_count").fetchone()[0]
page_size = cur.execute("PRAGMA page_size").fetchone()[0]
print(f'\nActual DB File Size: {(page_count * page_size) / (1024*1024):.1f} MB')

# Look at indexes size via dbstat if possible, otherwise we can't easily break down index sizes without dbstat or sqlite3_analyzer.
try:
    print("\n--- Index Sizes ---")
    indexes = [r[0] for r in cur.execute("SELECT name FROM sqlite_master WHERE type='index'").fetchall()]
    print(f"Number of indexes: {len(indexes)}")
except Exception as e:
    pass
