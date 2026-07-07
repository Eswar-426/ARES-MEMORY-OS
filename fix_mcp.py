with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()
text = text.replace('(*store_arc).clone()', 'store_arc.clone()')
text = text.replace('n.node_type.to_string()', 'format!("{:?}", n.node_type)')
with open('crates/ares-mcp/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
