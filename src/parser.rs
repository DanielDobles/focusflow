/// Custom parser for HOI4 focus tree files
/// 
/// We use a simple string-based approach instead of jomini because:
/// 1. Focus files have regular structure
/// 2. We need to preserve raw text for complex HOI4 blocks  
/// 3. jomini is optimized for save files, not mod source files

use anyhow::{Context, Result};
use std::collections::HashSet;

use crate::model::{FocusNode, FocusTree, Shortcut, ValidationResult, ValidationError};

/// Parse a focus tree file from raw text
pub fn parse_focus_file(content: &str) -> Result<FocusTree> {
    // Normalize line endings
    let content = content.replace("\r\n", "\n");
    
    // Find the focus_tree block
    let tree_start = content.find("focus_tree")
        .context("No 'focus_tree' found in file")?;
    
    // Extract everything inside focus_tree = { ... }
    let brace_start = content[tree_start..]
        .find('{')
        .context("No opening brace for focus_tree")?;
    let brace_start = tree_start + brace_start;
    
    let content_inside = extract_brace_block(&content, brace_start)?;
    
    // Parse tree-level fields
    let tree_id = extract_simple_string(&content_inside, "id").unwrap_or_else(|| "unknown".to_string());
    let shortcuts = parse_shortcuts(&content_inside);
    let focuses = parse_focuses(&content_inside);
    
    if focuses.is_empty() {
        anyhow::bail!("No focus nodes found");
    }
    
    Ok(FocusTree {
        id: tree_id,
        shortcuts,
        focuses,
        modified: false,
    })
}

/// Extract content inside a brace block (handling nesting)
fn extract_brace_block(content: &str, start: usize) -> Result<String> {
    let mut depth = 1;
    let bytes = content.as_bytes();
    let mut pos = start + 1; // Skip opening {
    
    while pos < bytes.len() && depth > 0 {
        match bytes[pos] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'#' => {
                // Skip comment lines
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
                continue;
            }
            b'"' => {
                // Skip quoted strings
                pos += 1;
                while pos < bytes.len() && bytes[pos] != b'"' {
                    if bytes[pos] == b'\\' {
                        pos += 1; // Skip escape
                    }
                    pos += 1;
                }
            }
            _ => {}
        }
        pos += 1;
    }
    
    Ok(content[start + 1..pos - 1].to_string())
}

/// Parse all focus blocks from the tree content
fn parse_focuses(content: &str) -> Vec<FocusNode> {
    let mut focuses = Vec::new();
    
    // Process line by line looking for "focus = {"
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let trimmed = lines[i].trim();
        
        // Check if this line starts "focus = {" or "focus ="
        let is_focus = trimmed.starts_with("focus =") || trimmed.starts_with("focus=");
        
        if is_focus && trimmed.contains('{') {
            // Found a focus block start
            // Reconstruct the block starting from the opening brace
            let mut block_text = String::new();
            let mut depth = 0;
            let mut started = false;
            
            // Collect lines until we find the matching closing brace
            for j in i..lines.len() {
                for ch in lines[j].chars() {
                    if ch == '{' {
                        depth += 1;
                        started = true;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                    
                    if started && depth == 0 {
                        // Found the end of the block
                        if let Some(focus) = parse_single_focus(&block_text) {
                            focuses.push(focus);
                        }
                        i = j + 1;
                        break;
                    }
                    block_text.push(ch);
                }
                
                if depth == 0 && started {
                    break;
                }
                
                block_text.push('\n');
            }
            
            if depth != 0 {
                // Unclosed block - move on
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    
    focuses
}

/// Parse a single focus from its block content
fn parse_single_focus(content: &str) -> Option<FocusNode> {
    let mut node = FocusNode {
        id: String::new(),
        icon: None,
        x: 0,
        y: 0,
        relative_position_id: None,
        cost: None,
        prerequisites: Vec::new(),
        mutually_exclusive: Vec::new(),
        bypass_if_unavailable: false,
        available_raw: None,
        completion_reward_raw: None,
        immediate_raw: None,
        ai_will_do_raw: None,
        search_filters: Vec::new(),
        bypass_raw: None,
    };
    
    // Extract simple fields
    if let Some(id) = extract_simple_string(content, "id") {
        node.id = id;
    }
    
    if node.id.is_empty() {
        return None; // Skip invalid focuses
    }
    
    node.icon = extract_simple_string(content, "icon");
    node.x = extract_simple_int(content, "x");
    node.y = extract_simple_int(content, "y");
    node.relative_position_id = extract_simple_string(content, "relative_position_id");
    node.cost = extract_simple_float(content, "cost");
    
    // Boolean field
    let bypass_val = extract_simple_string(content, "bypass_if_unavailable")
        .unwrap_or_default();
    node.bypass_if_unavailable = bypass_val == "yes" || bypass_val == "true";
    
    // List fields: prerequisite
    node.prerequisites = extract_list_field(content, "prerequisite", "focus");
    
    // List fields: mutually_exclusive
    node.mutually_exclusive = extract_list_field(content, "mutually_exclusive", "focus");
    
    // Search filters (can be bare values in a block)
    node.search_filters = extract_search_filters(content);
    
    // Complex blocks - extract as raw text
    node.available_raw = extract_block_as_text(content, "available");
    node.completion_reward_raw = extract_block_as_text(content, "completion_reward");
    node.immediate_raw = extract_block_as_text(content, "immediate");
    node.ai_will_do_raw = extract_block_as_text(content, "ai_will_do");
    node.bypass_raw = extract_block_as_text(content, "bypass");
    
    Some(node)
}

/// Extract a simple key = value (string)
fn extract_simple_string(content: &str, key: &str) -> Option<String> {
    // Search for the key with any amount of leading whitespace
    // Pattern: \n<whitespace>key<whitespace>=<whitespace>value
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", key)) || trimmed.starts_with(&format!("{}=", key)) {
            // Extract value after =
            if let Some(eq_pos) = trimmed.find('=') {
                let value = trimmed[eq_pos + 1..].trim();
                let value = value.trim_matches('"');
                if !value.is_empty() && !value.contains('{') {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Extract a simple key = value (integer)
fn extract_simple_int(content: &str, key: &str) -> i32 {
    if let Some(val) = extract_simple_string(content, key) {
        return val.parse().unwrap_or(0);
    }
    0
}

/// Extract a simple key = value (float)
fn extract_simple_float(content: &str, key: &str) -> Option<f32> {
    extract_simple_string(content, key).and_then(|v| v.parse().ok())
}

/// Extract list items from repeated blocks like:
/// prerequisite = { focus = A }
/// prerequisite = { focus = B }
fn extract_list_field(content: &str, block_key: &str, inner_key: &str) -> Vec<String> {
    let mut results = Vec::new();
    
    // Find all lines that start with the block_key
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", block_key)) || trimmed.starts_with(&format!("{}=", block_key)) {
            // Extract content between { and }
            if let Some(open) = trimmed.find('{') {
                if let Some(close) = trimmed.find('}') {
                    let inner = &trimmed[open + 1..close];
                    if let Some(value) = extract_simple_string(inner, inner_key) {
                        results.push(value);
                    }
                }
            }
        }
    }
    
    results
}

/// Extract search filters (special case: values in a block without explicit keys)
fn extract_search_filters(content: &str) -> Vec<String> {
    let mut filters = Vec::new();
    
    // Find the line with search_filters
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("search_filters") && trimmed.contains('{') {
            // Extract content between { and }
            if let Some(open) = trimmed.find('{') {
                if let Some(close) = trimmed.find('}') {
                    let inner = &trimmed[open + 1..close];
                    for token in inner.split_whitespace() {
                        if !token.is_empty() && !token.contains('=') {
                            filters.push(token.to_string());
                        }
                    }
                }
            }
            break;
        }
    }
    
    filters
}

/// Extract a block (key = { ... }) as raw text for preservation
/// Handles both single-line and multi-line blocks
fn extract_block_as_text(content: &str, key: &str) -> Option<String> {
    // Find the line that starts with the key
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", key)) || trimmed.starts_with(&format!("{}=", key)) {
            // Check if the block is on a single line
            if trimmed.contains('{') && trimmed.contains('}') {
                let open = trimmed.find('{').unwrap();
                let close = trimmed.rfind('}').unwrap();
                let inner = &trimmed[open + 1..close];
                return Some(format!("{{\n\t\t\t{}\n\t\t}}", inner.trim()));
            }
            
            // Multi-line block: collect actual lines until braces balance
            let mut block_lines = Vec::new();
            let mut depth = 0;
            let mut started = false;

            for subsequent_line in content.lines().skip(line_idx) {
                // Count braces in this line
                for ch in subsequent_line.chars() {
                    if ch == '{' { depth += 1; started = true; }
                    else if ch == '}' { depth -= 1; }
                }
                
                if started {
                    block_lines.push(subsequent_line.to_string());
                }
                
                if started && depth == 0 {
                    // Block ended
                    // Remove first { from first line and last } from last line
                    if let Some(first) = block_lines.first_mut() {
                        if let Some(pos) = first.find('{') {
                            *first = first[pos + 1..].to_string();
                        }
                    }
                    if let Some(last) = block_lines.last_mut() {
                        if let Some(pos) = last.rfind('}') {
                            *last = last[..pos].to_string();
                        }
                    }
                    return Some(format!("{{\n\t\t\t{}\n\t\t}}", block_lines.join("\n").trim()));
                }
            }
            
            return None; // Unclosed brace
        }
    }
    None
}

/// Parse shortcut definitions
fn parse_shortcuts(content: &str) -> Vec<Shortcut> {
    let mut shortcuts = Vec::new();
    let mut search_start = 0;
    
    while let Some(pos) = content[search_start..].find("shortcut = {") {
        let abs_start = search_start + pos + "shortcut = ".len();
        
        if let Ok(block_content) = extract_brace_block(content, abs_start) {
            let name = extract_simple_string(&block_content, "name").unwrap_or_default();
            let target = extract_simple_string(&block_content, "target").unwrap_or_default();
            
            if !target.is_empty() {
                shortcuts.push(Shortcut { name, target });
            }
        }
        
        search_start = abs_start + 1;
    }
    
    shortcuts
}

/// Validate a focus tree for common issues
pub fn validate_tree(tree: &FocusTree) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    let all_ids: HashSet<&str> = tree.focuses.iter().map(|f| f.id.as_str()).collect();
    
    // Check for duplicate IDs
    let mut seen_ids = HashSet::new();
    for focus in &tree.focuses {
        if seen_ids.contains(focus.id.as_str()) {
            errors.push(ValidationError::DuplicateId(focus.id.clone()));
        }
        seen_ids.insert(focus.id.as_str());
    }
    
    // Validate each focus
    for focus in &tree.focuses {
        // Check ID format (should have TAG_ prefix)
        if !focus.id.contains('_') || focus.id.len() < 4 {
            errors.push(ValidationError::InvalidIdFormat(focus.id.clone()));
        }
        
        // Check prerequisites exist
        for prereq in &focus.prerequisites {
            if !all_ids.contains(prereq.as_str()) {
                errors.push(ValidationError::MissingPrerequisite(prereq.clone()));
            }
        }
        
        // Check mutually_exclusive exist
        for me in &focus.mutually_exclusive {
            if !all_ids.contains(me.as_str()) {
                errors.push(ValidationError::MissingMutuallyExclusive(me.clone()));
            }
        }
        
        // Check cost range
        if let Some(cost) = focus.cost {
            if cost < 0.1 || cost > 100.0 {
                errors.push(ValidationError::UnusualCost { cost, min: 0.1, max: 100.0 });
            } else if cost < 1.0 || cost > 50.0 {
                warnings.push(ValidationError::UnusualCost { cost, min: 1.0, max: 50.0 });
            }
        }
        
        // Check position bounds
        if focus.x < -50 || focus.x > 50 || focus.y < -10 || focus.y > 50 {
            errors.push(ValidationError::PositionOutOfBounds { x: focus.x, y: focus.y });
        }
    }
    
    ValidationResult { errors, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_focus() {
        let data = r#"focus_tree = {
            id = test_focus
            
            focus = {
                id = TEST_first_focus
                icon = generic_industry
                x = 5
                y = 0
                cost = 5
                
                completion_reward = {
                    add_political_power = 50
                }
                
                ai_will_do = {
                    factor = 10
                }
            }
        }"#;
        
        // Debug: test parse_focuses directly
        let tree = parse_focus_file(data).unwrap();
        assert_eq!(tree.id, "test_focus");
        assert_eq!(tree.focuses.len(), 1);
        
        let focus = &tree.focuses[0];
        assert_eq!(focus.id, "TEST_first_focus");
        assert_eq!(focus.x, 5);
        assert_eq!(focus.y, 0);
        assert_eq!(focus.cost, Some(5.0));
        assert!(focus.completion_reward_raw.is_some());
    }
    
    #[test]
    fn test_parse_prerequisites() {
        let data = r#"focus_tree = {
            id = test
            
            focus = {
                id = TEST_a
                x = 0
                y = 0
            }
            
            focus = {
                id = TEST_b
                x = 0
                y = 1
                prerequisite = { focus = TEST_a }
            }
        }"#;
        
        let tree = parse_focus_file(data).unwrap();
        assert_eq!(tree.focuses.len(), 2);
        assert_eq!(tree.focuses[1].prerequisites, vec!["TEST_a"]);
    }
    
    #[test]
    fn test_parse_real_venezuela_file() {
        let md_path = r"C:\Users\armon\Documents\Paradox Interactive\Hearts of Iron IV\mod\MD\common\national_focus\venezuela.txt";
        let content = std::fs::read_to_string(md_path).expect("Failed to read venezuela.txt");
        let tree = parse_focus_file(&content).expect("Failed to parse venezuela.txt");
        
        assert_eq!(tree.id, "venezuela_focus");
        assert!(tree.focuses.len() > 300, "Expected 300+ focuses, got {}", tree.focuses.len());
        
        let first = &tree.focuses[0];
        assert_eq!(first.id, "VEN_reap_the_fruits");
        assert_eq!(first.x, 0);
        assert_eq!(first.y, 0);
        assert!(first.cost.is_some());
        
        let industry = tree.focuses.iter().find(|f| f.id == "VEN_venezuelan_industry").unwrap();
        assert_eq!(industry.x, 55);
        assert_eq!(industry.y, 0);
    }

    // === INTEGRATION TESTS ===
    
    #[test]
    fn test_roundtrip_parse_write_reparse() {
        use crate::writer;
        let md_path = r"C:\Users\armon\Documents\Paradox Interactive\Hearts of Iron IV\mod\MD\common\national_focus\venezuela.txt";
        let content = std::fs::read_to_string(md_path).unwrap();
        let tree1 = parse_focus_file(&content).unwrap();
        let output = writer::write_focus_tree(&tree1);
        
        // Write output to temp file for inspection
        std::fs::write("C:\\Users\\armon\\DEV\\HOI4_MD_FT\\focusflow\\debug_output.txt", &output).unwrap();
        
        // Check if "focus = {" appears in output
        let has_focus_blocks = output.contains("focus = {");
        eprintln!("DEBUG: output len={}, has_focus_blocks={}", output.len(), has_focus_blocks);
        
        // Show first focus block
        if let Some(pos) = output.find("focus = {") {
            let snippet = &output[pos..(pos + 200).min(output.len())];
            eprintln!("DEBUG: first focus block preview:\n{}", snippet);
        }
        
        let tree2 = parse_focus_file(&output).unwrap();
        
        assert_eq!(tree1.focuses.len(), tree2.focuses.len());
        for (f1, f2) in tree1.focuses.iter().zip(tree2.focuses.iter()) {
            assert_eq!(f1.id, f2.id);
            assert_eq!(f1.x, f2.x);
            assert_eq!(f1.y, f2.y);
            assert_eq!(f1.cost, f2.cost);
            assert_eq!(f1.icon, f2.icon);
            assert_eq!(f1.prerequisites, f2.prerequisites);
        }
    }

    #[test]
    fn test_performance_parse_venezuela() {
        use std::time::Instant;
        let md_path = r"C:\Users\armon\Documents\Paradox Interactive\Hearts of Iron IV\mod\MD\common\national_focus\venezuela.txt";
        let content = std::fs::read_to_string(md_path).unwrap();
        
        let _ = parse_focus_file(&content); // warm up
        
        let iterations = 10;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parse_focus_file(&content);
        }
        let avg_ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
        println!("⚡ Parse avg: {:.2}ms", avg_ms);
        assert!(avg_ms < 200.0, "Parse too slow: {:.2}ms", avg_ms);
    }

    #[test]
    fn test_braces_balanced_in_output() {
        use crate::writer;
        let md_path = r"C:\Users\armon\Documents\Paradox Interactive\Hearts of Iron IV\mod\MD\common\national_focus\venezuela.txt";
        let content = std::fs::read_to_string(md_path).unwrap();
        let tree = parse_focus_file(&content).unwrap();
        let output = writer::write_focus_tree(&tree);
        
        let open = output.chars().filter(|&c| c == '{').count();
        let close = output.chars().filter(|&c| c == '}').count();
        assert_eq!(open, close, "Braces must be balanced: {} vs {}", open, close);
    }

    #[test]
    fn test_focus_categories_venezuela() {
        let md_path = r"C:\Users\armon\Documents\Paradox Interactive\Hearts of Iron IV\mod\MD\common\national_focus\venezuela.txt";
        let content = std::fs::read_to_string(md_path).unwrap();
        let tree = parse_focus_file(&content).unwrap();
        
        let mut categories = std::collections::HashMap::new();
        for f in &tree.focuses {
            *categories.entry(f.category().to_string()).or_insert(0) += 1;
        }
        
        assert!(categories.len() >= 3, "Need 3+ categories, got {}", categories.len());
        let with_icon = tree.focuses.iter().filter(|f| f.icon.is_some()).count();
        let with_cost = tree.focuses.iter().filter(|f| f.cost.is_some()).count();
        println!("Categories: {:?} | Icons: {}/{} | Costs: {}/{}", 
            categories, with_icon, tree.focuses.len(), with_cost, tree.focuses.len());
    }
}
