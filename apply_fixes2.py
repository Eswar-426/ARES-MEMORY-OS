with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('e.edge_type == "depends_on"', 'e.edge_type.as_str() == "depends_on"')

with open('crates/ares-mcp/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
