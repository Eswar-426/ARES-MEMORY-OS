import subprocess
p = subprocess.Popen(
    ["C:\\Users\\eswar\\.antigravity-ide\\extensions\\eswar426.ares-memory-vscode-0.1.0\\binaries\\windows\\ares-mcp.exe"],
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    cwd="E:\\My Projects\\ARES_Memory_os"
)
try:
    out, err = p.communicate(timeout=2)
except subprocess.TimeoutExpired:
    p.kill()
    out, err = p.communicate()
print("STDOUT:")
print(out)
print("STDERR:")
print(err)
