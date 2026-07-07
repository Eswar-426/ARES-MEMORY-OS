import re
with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()
matches = re.findall(r'ToolBuilder::new\("(.*?)"\)', text)
for m in matches:
    print(m)
