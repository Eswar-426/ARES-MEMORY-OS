const { spawn } = require('child_process');

const mcp = spawn('.\\target\\debug\\ares-mcp.exe');

let id = 1;
function sendRequest(method, params) {
    const req = {
        jsonrpc: '2.0',
        id: id++,
        method,
        params
    };
    mcp.stdin.write(JSON.stringify(req) + '\n');
}

mcp.stdout.on('data', (data) => {
    console.log(`[MCP RESPONSE]: ${data.toString()}`);
});

mcp.stderr.on('data', (data) => {
    console.error(`[MCP ERR]: ${data.toString()}`);
});

// Send initialize
sendRequest('initialize', {
    protocolVersion: '2024-11-05',
    capabilities: {},
    clientInfo: { name: 'test', version: '1.0' }
});

setTimeout(() => {
    sendRequest('notifications/initialized', {});
    
    // Call Why Exists on crates/ares-cli/src/main.rs
    sendRequest('tools/call', {
        name: 'ares_why_exists',
        arguments: { id: 'crates/ares-cli/src/main.rs' }
    });

    sendRequest('tools/call', {
        name: 'ares_why_exists',
        arguments: { id: 'extensions/ares-memory-vscode/src/extension.ts' }
    });

    sendRequest('tools/call', {
        name: 'ares_why_exists',
        arguments: { id: 'crates/ares-ingestion/src/graph.rs' }
    });

    sendRequest('tools/call', {
        name: 'ares_impact',
        arguments: { id: 'crates/ares-core/Cargo.toml' }
    });

    sendRequest('tools/call', {
        name: 'ares_simulate',
        arguments: { project_id: 'PROJ-001', action: 'remove', target_id: 'crates/ares-core/Cargo.toml' }
    });

}, 1000);

setTimeout(() => {
    mcp.kill();
}, 3000);
