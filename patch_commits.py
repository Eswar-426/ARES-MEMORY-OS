import re

with open('crates/ares-git-memory/src/commits.rs', 'r', encoding='utf-8') as f:
    text = f.read()

struct_code = """
#[derive(Debug, Clone)]
pub struct PrDecision {
    pub pr_number: Option<i64>,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub extracted_heading: Option<String>,
    pub commit_hash: String,
    pub touched_files: Vec<String>,
}

pub fn extract_pr_decision(subject: &str, body: &str, hash: &str, files: &[String]) -> Option<PrDecision> {
    let mut pr_number = None;
    
    let re_squash = regex::Regex::new(r"^(.+?)\\s*\\(#(\\d+)\\)$").unwrap();
    let re_merge = regex::Regex::new(r"^Merge pull request #(\\d+)").unwrap();
    
    let title;
    if let Some(caps) = re_squash.captures(subject) {
        title = caps.get(1).unwrap().as_str().to_string();
        pr_number = caps.get(2).unwrap().as_str().parse::<i64>().ok();
    } else if let Some(caps) = re_merge.captures(subject) {
        title = subject.to_string();
        pr_number = caps.get(1).unwrap().as_str().parse::<i64>().ok();
    } else {
        return None;
    }

    let headings = vec![
        "## why", "## context", "## decision", "## motivation", "## background", 
        "## problem", "## solution", "## approach", "## rationale", "## tradeoffs",
        "### why", "### context", "### decision", "### motivation", "### background", 
        "### problem", "### solution", "### approach", "### rationale", "### tradeoffs"
    ];
    
    let mut extracted_heading = None;
    let mut extracted_text = String::new();
    
    let mut in_target_heading = false;
    for line in body.lines() {
        let lower = line.trim().to_lowercase();
        if lower.starts_with("# ") || lower.starts_with("## ") || lower.starts_with("### ") {
            if headings.contains(&lower.as_str()) {
                in_target_heading = true;
                extracted_heading = Some(line.trim().to_string());
                continue;
            } else if in_target_heading {
                break; // next heading
            }
        }
        if in_target_heading {
            extracted_text.push_str(line);
            extracted_text.push('\\n');
        }
    }
    
    let mut confidence = 0.0;
    
    if extracted_heading.is_some() {
        confidence = 0.8;
    } else {
        let keywords = vec![
            "because", "instead of", "chose", "decided", "reason", "motivated by",
            "alternative", "tradeoff", "trade-off", "we chose", "the goal"
        ];
        let lower_body = body.to_lowercase();
        let mut hit_count = 0;
        for kw in keywords {
            hit_count += lower_body.matches(kw).count();
        }
        if hit_count > 0 {
            confidence = (hit_count as f64 * 0.15).min(1.0);
            extracted_text = body.to_string();
        }
    }
    
    if confidence < 0.4 {
        return None;
    }
    
    Some(PrDecision {
        pr_number,
        title,
        description: extracted_text.trim().to_string(),
        confidence,
        extracted_heading,
        commit_hash: hash.to_string(),
        touched_files: files.to_vec(),
    })
}
"""

idx = text.find('pub struct CommitExtractor;')
text = text[:idx] + struct_code + '\n' + text[idx:]

text = text.replace(
    'pub fn extract(',
    'pub fn extract('
).replace(
    ') -> Result<(Vec<GraphNode>, Vec<GraphEdge>), String> {',
    ') -> Result<(Vec<GraphNode>, Vec<GraphEdge>, Vec<PrDecision>), String> {'
)

text = text.replace(
    '        let mut nodes = Vec::new();\n        let mut edges = Vec::new();',
    '        let mut nodes = Vec::new();\n        let mut edges = Vec::new();\n        let mut pr_decisions = Vec::new();'
)

text = text.replace(
    '        Ok((nodes, edges))\n    }',
    '        Ok((nodes, edges, pr_decisions))\n    }'
)

files_extract = """            let mut files_list = Vec::new();
            for line in files_part.lines() {
                if line.is_empty() { continue; }
                let f_parts: Vec<&str> = line.split('\\t').collect();
                if f_parts.len() >= 2 {
                    let status = f_parts[0];
                    let file_path = if status.starts_with('R') && f_parts.len() >= 3 { f_parts[2] } else { f_parts[1] };
                    files_list.push(file_path.to_string());
                }
            }
"""

idx3 = text.find('            // 4. Extract Decisions')
text = text[:idx3] + files_extract + '\n' + text[idx3:]

pr_decision_append = """
            if let Some(pr_dec) = extract_pr_decision(subject, body, hash, &files_list) {
                pr_decisions.push(pr_dec);
            }
"""

idx4 = text.find('            // 5. Process Touched Files')
text = text[:idx4] + pr_decision_append + '\n' + text[idx4:]

with open('crates/ares-git-memory/src/commits.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print('Updated commits.rs successfully')
