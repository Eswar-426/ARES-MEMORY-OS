import subprocess
import json

p = subprocess.Popen(
    ["C:\\Users\\eswar\\.antigravity-ide\\extensions\\eswar426.ares-memory-vscode-0.1.0\\binaries\\windows\\ares-mcp.exe"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
    cwd="E:\\My Projects\\ARES_Memory_os"
)

req = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test", "version": "1.0"}
    }
}
p.stdin.write(json.dumps(req) + "\n")
p.stdin.flush()

out = p.stdout.readline()
print("OUT:", out)
p.kill()
