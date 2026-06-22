import sqlite3
conn = sqlite3.connect('memory.db')
cursor = conn.cursor()
print(cursor.execute("SELECT name, sql FROM sqlite_master WHERE type='table' AND name='graph_nodes'").fetchall())
