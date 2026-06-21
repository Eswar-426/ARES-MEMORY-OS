import subprocess
import json
import time

p = subprocess.Popen(['target/release/ares-mcp.exe'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
req = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'initialize', 'params': {'protocolVersion': '2024-11-05', 'capabilities': {}, 'clientInfo': {'name': 'test', 'version': '1.0'}}})
p.stdin.write(req + '\n')
p.stdin.flush()

print(p.stdout.readline())

req2 = json.dumps({'jsonrpc': '2.0', 'id': 2, 'method': 'tools/call', 'params': {'name': 'ares_why_exists', 'arguments': {'id': 'crates/ares-cli/src/main.rs'}}})
p.stdin.write(req2 + '\n')
p.stdin.flush()

print(p.stdout.readline())
p.terminate()
