with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Apply the replacements manually in Python so we don't have to worry about whitespace
text = text.replace('get_edges_to_by_type(&file_id, "OwnedBy")', 'get_edges_to_by_type(&file_id, "authored_by")')
text = text.replace('get_edges_to_by_type(&file_id, "ContributedTo")', 'get_edges_to_by_type(&file_id, "contributed_to")')
text = text.replace('get_edges_to_by_type(&file_id, "Touches")', 'get_edges_to_by_type(&file_id, "touches")')

text = text.replace('e.edge_type.as_str().contains("DependsOn") || e.edge_type.as_str().contains("depends")', 'e.edge_type == "depends_on"')

text = text.replace('n.node_type == ares_core::NodeType::File', 'format!("{:?}", n.node_type).to_lowercase() == "file"')

# For architecture types counting, we had `format!("{:?}", n.node_type)`
# we probably want `.to_lowercase()` added to it if it's not already there.
# Let's check: 
text = text.replace('type_counts.entry(format!("{:?}", n.node_type)).or_insert(0) += 1;', 'type_counts.entry(format!("{:?}", n.node_type).to_lowercase()).or_insert(0) += 1;')

with open('crates/ares-mcp/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
