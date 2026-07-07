with open('crates/ares-mcp/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

replacement = """let project_name = std::path::Path::new(&pp).file_name().unwrap_or_default().to_string_lossy().to_string();
                let project_id = ares_core::ProjectId::from(project_name);"""

text = text.replace('let project_id = ares_core::ProjectId::from(pp);', replacement)
text = text.replace('let _project_id = ares_core::ProjectId::from(pp);', replacement.replace('let project_id', 'let _project_id'))

with open('crates/ares-mcp/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
