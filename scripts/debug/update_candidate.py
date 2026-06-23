import sys

with open('crates/ares-store/src/repositories/candidate.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace imports to make sure ArchitectureCategory is there
if 'ArchitectureCategory,' not in content:
    content = content.replace('DecisionCategory,', 'DecisionCategory, ArchitectureCategory,')

# Add architecture_category match helper
arch_match = """
        let architecture_category_str = match &candidate.architecture_category {
            Some(ArchitectureCategory::Service) => Some("Service"),
            Some(ArchitectureCategory::Component) => Some("Component"),
            Some(ArchitectureCategory::Module) => Some("Module"),
            Some(ArchitectureCategory::Workspace) => Some("Workspace"),
            Some(ArchitectureCategory::Domain) => Some("Domain"),
            Some(ArchitectureCategory::Integration) => Some("Integration"),
            None => None,
        };

        let dependent_components_str = candidate.dependent_components.join(",");
        let ownership_domains_str = candidate.ownership_domains.join(",");
"""

# INSERT INTO
if "let architecture_category_str" not in content:
    content = content.replace(
        'let decision_category_str = match &candidate.decision_category {',
        arch_match + '\n        let decision_category_str = match &candidate.decision_category {'
    )

    content = content.replace(
        'created_at, updated_at, decision_category\n            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)',
        'created_at, updated_at, decision_category, architecture_category, dependent_components, ownership_domains\n            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)'
    )

    content = content.replace(
        'candidate.updated_at,\n                decision_category_str,\n            ],',
        'candidate.updated_at,\n                decision_category_str,\n                architecture_category_str,\n                dependent_components_str,\n                ownership_domains_str,\n            ],'
    )

# SELECT queries
content = content.replace(
    'created_at, updated_at, decision_category\n                 FROM',
    'created_at, updated_at, decision_category, architecture_category, dependent_components, ownership_domains\n                 FROM'
)

# Row decoding
decode_block = """
                    let arch_cat_str: Option<String> = row.get(13)?;
                    let architecture_category = arch_cat_str.and_then(|s| match s.as_str() {
                        "Service" => Some(ArchitectureCategory::Service),
                        "Component" => Some(ArchitectureCategory::Component),
                        "Module" => Some(ArchitectureCategory::Module),
                        "Workspace" => Some(ArchitectureCategory::Workspace),
                        "Domain" => Some(ArchitectureCategory::Domain),
                        "Integration" => Some(ArchitectureCategory::Integration),
                        _ => None,
                    });

                    let dep_comp: String = row.get(14).unwrap_or_default();
                    let owner_dom: String = row.get(15).unwrap_or_default();
                    let dependent_components = if dep_comp.is_empty() { vec![] } else { dep_comp.split(",").map(|s| s.to_string()).collect() };
                    let ownership_domains = if owner_dom.is_empty() { vec![] } else { owner_dom.split(",").map(|s| s.to_string()).collect() };
"""

# Make sure we don't insert it multiple times.
if "let arch_cat_str:" not in content:
    content = content.replace(
        'Ok(Candidate {\n                        id: row.get(0)?,',
        decode_block + '\n                    Ok(Candidate {\n                        id: row.get(0)?,'
    )
    content = content.replace(
        'Ok(Candidate {\n                    id: row.get(0)?,',
        decode_block + '\n                    Ok(Candidate {\n                    id: row.get(0)?,'
    )

# Update fields in the OK candidate struct
# In case it has been partially replaced by the multi_replace tool:
content = content.replace(
    'decision_category,\n                        status,',
    'decision_category,\n                        architecture_category,\n                        dependent_components,\n                        ownership_domains,\n                        status,'
)
content = content.replace(
    'decision_category,\n                    architecture_category: row.get(13)?,\n                    dependent_components: if dep_comp.is_empty() { vec![] } else { dep_comp.split(\',\').map(|s| s.to_string()).collect() },\n                    ownership_domains: if owner_dom.is_empty() { vec![] } else { owner_dom.split(\',\').map(|s| s.to_string()).collect() },\n                    status,',
    'decision_category,\n                    architecture_category,\n                    dependent_components,\n                    ownership_domains,\n                    status,'
)

# UPDATE candidates
content = content.replace(
    'updated_at = ?11, decision_category = ?12\n             WHERE id = ?1',
    'updated_at = ?11, decision_category = ?12, architecture_category = ?13, dependent_components = ?14, ownership_domains = ?15\n             WHERE id = ?1'
)

with open('crates/ares-store/src/repositories/candidate.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("Updated candidate.rs successfully")
