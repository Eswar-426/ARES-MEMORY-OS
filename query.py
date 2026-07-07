import sqlite3
import json

conn = sqlite3.connect('.ares/ares.db')
cursor = conn.cursor()
cursor.execute("SELECT properties FROM graph_nodes WHERE node_type='file' LIMIT 1")
print(cursor.fetchone()[0])
