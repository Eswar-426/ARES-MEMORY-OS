import subprocess
import json
import time
import statistics
import os
import sys

REPOS = [
    {"name": "ARES", "path": ".", "target": "crates/ares-cli/src/main.rs"},
    {"name": "Automyra", "path": ".temp/automyra", "target": "src/main.rs"},
    {"name": "ripgrep", "path": ".temp/ripgrep", "target": "crates/core/main.rs"},
    {"name": "Next.js", "path": ".temp/nextjs", "target": "packages/next/src/server/next.ts"}
]

COMMANDS = ["Doctor", "Ingest", "Why Exists", "Impact", "Coverage", "Evolution", "Simulate"]
RUNS = 3

def json_rpc(method, params):
    return json.dumps({
        "jsonrpc": "2.0",
        "id": 2,
        "method": method,
        "params": params
    }) + "\n"

def measure_mcp(mcp_path, cwd, method, params):
    latencies = []
    successes = 0
    errors = 0
    for _ in range(RUNS):
        start = time.time()
        try:
            p = subprocess.Popen([mcp_path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=cwd, text=True)
            p.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'initialize', 'params': {'protocolVersion': '2024-11-05', 'capabilities': {}, 'clientInfo': {'name': 'test', 'version': '1.0'}}}) + '\n')
            p.stdin.flush()
            p.stdout.readline() # Read initialize response
            p.stdin.write(json_rpc("tools/call", {
                "name": method,
                "arguments": params
            }))
            p.stdin.flush()
            
            # Read line by line until we get the response
            response = None
            while True:
                line = p.stdout.readline()
                if not line:
                    break
                try:
                    msg = json.loads(line)
                    if msg.get("id") == 2:
                        response = msg
                        break
                except json.JSONDecodeError:
                    pass
            
            p.terminate()
            
            latencies.append((time.time() - start) * 1000)
            if response and "result" in response and not response.get("result", {}).get("isError", False):
                successes += 1
            else:
                errors += 1
        except Exception as e:
            errors += 1
            latencies.append((time.time() - start) * 1000)
            
    if not latencies:
        return 0, 0, 0, 0, RUNS
    
    p50 = statistics.median(latencies)
    p95 = statistics.quantiles(latencies, n=20)[18] if len(latencies) >= 20 else max(latencies)
    return (successes / RUNS) * 100, p50, p95, errors, 0

def measure_cli(exe_path, cwd, args):
    latencies = []
    successes = 0
    errors = 0
    for _ in range(RUNS):
        start = time.time()
        try:
            result = subprocess.run([exe_path] + args, cwd=cwd, capture_output=True, timeout=30)
            latencies.append((time.time() - start) * 1000)
            if result.returncode == 0:
                successes += 1
            else:
                errors += 1
        except subprocess.TimeoutExpired:
            errors += 1
            latencies.append((time.time() - start) * 1000)
        except Exception:
            errors += 1
            latencies.append((time.time() - start) * 1000)
            
    if not latencies:
        return 0, 0, 0, 0, RUNS
    
    p50 = statistics.median(latencies)
    p95 = max(latencies)
    return (successes / RUNS) * 100, p50, p95, errors, 0

def main():
    ares_exe = os.path.abspath("target/release/ares.exe")
    mcp_exe = os.path.abspath("target/release/ares-mcp.exe")
    
    print("# Extension Certification Report")
    print("Evaluating MCP and CLI responsiveness for Extension Integration.\n")
    
    for cmd in COMMANDS:
        print(f"## {cmd}")
        print("| Repository | Success % | P50 (ms) | P95 (ms) | Errors | Timeouts |")
        print("|---|---|---|---|---|---|")
        
        for repo in REPOS:
            success, p50, p95, errors, timeouts = 0, 0, 0, 0, 0
            
            if not os.path.exists(repo["path"]):
                print(f"| {repo['name']} | skipped | - | - | - | - |")
                continue
                
            cwd = repo["path"]
            
            if cmd == "Doctor":
                success, p50, p95, errors, timeouts = measure_cli(ares_exe, cwd, ["doctor"])
            elif cmd == "Ingest":
                success, p50, p95, errors, timeouts = measure_cli(ares_exe, cwd, ["ingest", "."])
            elif cmd == "Why Exists":
                success, p50, p95, errors, timeouts = measure_mcp(mcp_exe, cwd, "ares_why_exists", {"id": repo["target"]})
            elif cmd == "Impact":
                success, p50, p95, errors, timeouts = measure_mcp(mcp_exe, cwd, "ares_impact", {"id": repo["target"]})
            elif cmd == "Coverage":
                success, p50, p95, errors, timeouts = measure_mcp(mcp_exe, cwd, "ares_coverage", {"project_id": "PROJ-1"})
            elif cmd == "Evolution":
                success, p50, p95, errors, timeouts = measure_mcp(mcp_exe, cwd, "ares_evolution", {"id": repo["target"]})
            elif cmd == "Simulate":
                success, p50, p95, errors, timeouts = measure_mcp(mcp_exe, cwd, "ares_simulate", {
                    "project_id": "PROJ-1", 
                    "action": "remove", 
                    "target_id": repo["target"]
                })
                
            print(f"| {repo['name']} | {success:.1f}% | {p50:.1f} | {p95:.1f} | {errors} | {timeouts} |")
        print("")

if __name__ == "__main__":
    main()
