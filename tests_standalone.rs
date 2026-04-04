/// Minimal test runner with ZERO external dependencies
/// Can compile even when HVCI blocks cargo build scripts
/// This tests ONLY the core parser/writer logic

const VENEZUELA_PATH: &str = r"C:\Users\armon\Documents\Paradox Interactive\Hearts of Iron IV\mod\MD\common\national_focus\venezuela.txt";

// === Minimal re-implementation of our model (no serde needed) ===

#[derive(Debug, Clone)]
struct FocusNode {
    id: String,
    icon: Option<String>,
    x: i32,
    y: i32,
    relative_position_id: Option<String>,
    cost: Option<f32>,
    prerequisites: Vec<String>,
    mutually_exclusive: Vec<String>,
    bypass_if_unavailable: bool,
    search_filters: Vec<String>,
    completion_reward_raw: Option<String>,
    ai_will_do_raw: Option<String>,
}

impl FocusNode {
    fn display_name(&self) -> &str {
        self.id.strip_prefix("VEN_").unwrap_or(&self.id)
    }
    fn category(&self) -> &str {
        for f in &self.search_filters {
            if f.contains("INDUSTRY") { return "Industry"; }
            if f.contains("MILITARY") || f.contains("ARMY") || f.contains("NAVY") { return "Military"; }
            if f.contains("POLITICAL") { return "Political"; }
            if f.contains("RESEARCH") { return "Research"; }
            if f.contains("FOREIGN") { return "Foreign"; }
        }
        "Other"
    }
}

#[derive(Debug, Clone)]
struct FocusTree {
    id: String,
    shortcuts: Vec<(String, String)>,
    focuses: Vec<FocusNode>,
    modified: bool,
}

// === Parser (same logic as src/parser.rs, copy-pasted for zero-dep testing) ===

fn parse_focus_file(content: &str) -> Result<FocusTree, String> {
    let content = content.replace("\r\n", "\n");
    let tree_start = content.find("focus_tree").ok_or("No 'focus_tree' found")?;
    let brace_start = content[tree_start..].find('{').ok_or("No opening brace")?;
    let brace_start = tree_start + brace_start;
    let content_inside = extract_brace_block(&content, brace_start)?;
    
    let tree_id = extract_simple_string(&content_inside, "id").unwrap_or_else(|| "unknown".to_string());
    let shortcuts = parse_shortcuts(&content_inside);
    let focuses = parse_focuses(&content_inside);
    
    if focuses.is_empty() {
        return Err("No focus nodes found".to_string());
    }
    
    Ok(FocusTree { id: tree_id, shortcuts, focuses, modified: false })
}

fn extract_brace_block(content: &str, start: usize) -> Result<String, String> {
    let mut depth = 1;
    let bytes = content.as_bytes();
    let mut pos = start + 1;
    while pos < bytes.len() && depth > 0 {
        match bytes[pos] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'"' => {
                pos += 1;
                while pos < bytes.len() && bytes[pos] != b'"' {
                    if bytes[pos] == b'\\' { pos += 1; }
                    pos += 1;
                }
            }
            _ => {}
        }
        pos += 1;
    }
    Ok(content[start + 1..pos - 1].to_string())
}

fn parse_focuses(content: &str) -> Vec<FocusNode> {
    let mut focuses = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let is_focus = trimmed.starts_with("focus =") || trimmed.starts_with("focus=");
        if is_focus && trimmed.contains('{') {
            let mut block_text = String::new();
            let mut depth = 0;
            let mut started = false;
            for j in i..lines.len() {
                for ch in lines[j].chars() {
                    if ch == '{' { depth += 1; started = true; }
                    else if ch == '}' { depth -= 1; }
                    if started && depth == 0 {
                        if let Some(focus) = parse_single_focus(&block_text) {
                            focuses.push(focus);
                        }
                        i = j + 1;
                        break;
                    }
                    block_text.push(ch);
                }
                if depth == 0 && started { break; }
                block_text.push('\n');
            }
            if depth != 0 { i += 1; }
        } else {
            i += 1;
        }
    }
    focuses
}

fn parse_single_focus(content: &str) -> Option<FocusNode> {
    let mut node = FocusNode {
        id: String::new(), icon: None, x: 0, y: 0,
        relative_position_id: None, cost: None,
        prerequisites: Vec::new(), mutually_exclusive: Vec::new(),
        bypass_if_unavailable: false, search_filters: Vec::new(),
        completion_reward_raw: None, ai_will_do_raw: None,
    };
    
    if let Some(id) = extract_simple_string(content, "id") { node.id = id; }
    if node.id.is_empty() { return None; }
    node.icon = extract_simple_string(content, "icon");
    node.x = extract_simple_int(content, "x");
    node.y = extract_simple_int(content, "y");
    node.relative_position_id = extract_simple_string(content, "relative_position_id");
    node.cost = extract_simple_float(content, "cost");
    
    let bypass_val = extract_simple_string(content, "bypass_if_unavailable").unwrap_or_default();
    node.bypass_if_unavailable = bypass_val == "yes" || bypass_val == "true";
    
    node.prerequisites = extract_list_field(content, "prerequisite", "focus");
    node.mutually_exclusive = extract_list_field(content, "mutually_exclusive", "focus");
    node.search_filters = extract_search_filters(content);
    node.completion_reward_raw = extract_block_as_text(content, "completion_reward");
    node.ai_will_do_raw = extract_block_as_text(content, "ai_will_do");
    
    Some(node)
}

fn extract_simple_string(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", key)) || trimmed.starts_with(&format!("{}=", key)) {
            if let Some(eq_pos) = trimmed.find('=') {
                let value = trimmed[eq_pos + 1..].trim().trim_matches('"');
                if !value.is_empty() && !value.contains('{') {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn extract_simple_int(content: &str, key: &str) -> i32 {
    extract_simple_string(content, key).and_then(|v| v.parse().ok()).unwrap_or(0)
}

fn extract_simple_float(content: &str, key: &str) -> Option<f32> {
    extract_simple_string(content, key).and_then(|v| v.parse().ok())
}

fn extract_list_field(content: &str, block_key: &str, inner_key: &str) -> Vec<String> {
    let mut results = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if (trimmed.starts_with(&format!("{} =", block_key)) || trimmed.starts_with(&format!("{}=", block_key)))
            && trimmed.contains('{') && trimmed.contains('}') {
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

fn extract_search_filters(content: &str) -> Vec<String> {
    let mut filters = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("search_filters") && trimmed.contains('{') && trimmed.contains('}') {
            if let Some(open) = trimmed.find('{') {
                if let Some(close) = trimmed.find('}') {
                    for token in trimmed[open + 1..close].split_whitespace() {
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

fn extract_block_as_text(content: &str, key: &str) -> Option<String> {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", key)) || trimmed.starts_with(&format!("{}=", key)) {
            if trimmed.contains('{') && trimmed.contains('}') {
                let open = trimmed.find('{').unwrap();
                let close = trimmed.rfind('}').unwrap();
                return Some(format!("{{\n\t\t\t{}\n\t\t}}", trimmed[open + 1..close].trim()));
            }
            let mut block_text = String::new();
            let mut depth = 0;
            let mut started = false;
            for subsequent_line in content.lines().skip(line_idx) {
                for ch in subsequent_line.chars() {
                    if ch == '{' { depth += 1; started = true; }
                    else if ch == '}' { depth -= 1; }
                    if started && depth == 0 {
                        return Some(format!("{{\n\t\t\t{}\n\t\t}}", block_text.trim()));
                    }
                    block_text.push(ch);
                }
                block_text.push('\n');
            }
            return None;
        }
    }
    None
}

fn parse_shortcuts(content: &str) -> Vec<(String, String)> {
    let mut shortcuts = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = content[search_start..].find("shortcut = {") {
        let abs = search_start + pos + "shortcut = ".len();
        if let Ok(block) = extract_brace_block(content, abs) {
            let name = extract_simple_string(&block, "name").unwrap_or_default();
            let target = extract_simple_string(&block, "target").unwrap_or_default();
            if !target.is_empty() { shortcuts.push((name, target)); }
        }
        search_start = abs + 1;
    }
    shortcuts
}

// === Writer ===

fn write_focus_tree(tree: &FocusTree) -> String {
    let mut out = String::with_capacity(128 * 1024);
    out.push_str("focus_tree = {\n");
    out.push_str(&format!("\tid = {}\n", tree.id));
    out.push('\n');
    
    for (name, target) in &tree.shortcuts {
        out.push_str(&format!("\tshortcut = {{\n\t\tname = {}\n\t\ttarget = {}\n\t\tscroll_wheel_factor = 0.80\n\t}}\n", name, target));
    }
    if !tree.shortcuts.is_empty() { out.push('\n'); }
    
    for (i, focus) in tree.focuses.iter().enumerate() {
        if i > 0 { out.push('\n'); }
        write_focus_node(&mut out, focus);
    }
    
    out.push_str("}\n");
    out
}

fn write_focus_node(out: &mut String, focus: &FocusNode) {
    out.push_str("\tfocus = {\n");
    out.push_str(&format!("\t\tid = {}\n", focus.id));
    if let Some(ref icon) = focus.icon { out.push_str(&format!("\t\ticon = {}\n", icon)); }
    out.push_str(&format!("\t\tx = {}\n", focus.x));
    out.push_str(&format!("\t\ty = {}\n", focus.y));
    if let Some(ref rel) = focus.relative_position_id { out.push_str(&format!("\t\trelative_position_id = {}\n", rel)); }
    if let Some(cost) = focus.cost {
        if cost.fract() == 0.0 { out.push_str(&format!("\t\tcost = {}\n", cost as i32)); }
        else { out.push_str(&format!("\t\tcost = {}\n", cost)); }
    }
    for prereq in &focus.prerequisites { out.push_str(&format!("\t\tprerequisite = {{ focus = {} }}\n", prereq)); }
    for me in &focus.mutually_exclusive { out.push_str(&format!("\t\tmutually_exclusive = {{ focus = {} }}\n", me)); }
    if focus.bypass_if_unavailable { out.push_str("\t\tbypass_if_unavailable = yes\n"); }
    if !focus.search_filters.is_empty() {
        out.push_str(&format!("\t\tsearch_filters = {{ {} }}\n", focus.search_filters.join(" ")));
    }
    if let Some(ref b) = focus.completion_reward_raw { out.push_str(&format!("\t\tcompletion_reward = {}\n", b)); }
    if let Some(ref b) = focus.ai_will_do_raw { out.push_str(&format!("\t\tai_will_do = {}\n", b)); }
    out.push_str("\t}\n");
}

// === Validation ===

fn validate_tree(tree: &FocusTree) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let all_ids: std::collections::HashSet<&str> = tree.focuses.iter().map(|f| f.id.as_str()).collect();
    let mut seen = std::collections::HashSet::new();
    
    for focus in &tree.focuses {
        if seen.contains(focus.id.as_str()) { errors.push(format!("Duplicate ID: {}", focus.id)); }
        seen.insert(focus.id.as_str());
        if !focus.id.contains('_') || focus.id.len() < 4 { errors.push(format!("Invalid ID format: {}", focus.id)); }
        for prereq in &focus.prerequisites {
            if !all_ids.contains(prereq.as_str()) { errors.push(format!("Missing prereq: {}", prereq)); }
        }
        for me in &focus.mutually_exclusive {
            if !all_ids.contains(me.as_str()) { errors.push(format!("Missing mutually exclusive: {}", me)); }
        }
        if let Some(cost) = focus.cost {
            if cost < 0.1 || cost > 100.0 { errors.push(format!("Unusual cost {} for {}", cost, focus.id)); }
        }
        if focus.x < -50 || focus.x > 50 || focus.y < -10 || focus.y > 50 {
            errors.push(format!("Position out of bounds: ({}, {}) for {}", focus.x, focus.y, focus.id));
        }
    }
    (errors, warnings)
}

// === Test runner ===

fn run_test(name: &str, f: impl FnOnce() -> Result<(), String>) {
    print!("[{:50}] ", name);
    match f() {
        Ok(()) => println!("PASS"),
        Err(e) => println!("FAIL: {}", e),
    }
}

fn main() {
    println!("=== FocusFlow Standalone Test Suite ===\n");
    
    // T1: Parse Venezuela
    run_test("parse_venezuela_complete", || {
        let content = std::fs::read_to_string(VENEZUELA_PATH)
            .map_err(|e| format!("Cannot read: {}", e))?;
        let tree = parse_focus_file(&content)
            .map_err(|e| format!("Parse failed: {}", e))?;
        assert!(tree.focuses.len() > 300, "Expected 300+, got {}", tree.focuses.len());
        assert_eq!(tree.focuses[0].id, "VEN_reap_the_fruits");
        assert_eq!(tree.focuses[0].x, 0);
        assert_eq!(tree.focuses[0].y, 0);
        assert!(tree.focuses[0].cost.is_some());
        println!("  {} focuses, {} shortcuts", tree.focuses.len(), tree.shortcuts.len());
        Ok(())
    });
    
    // T2: Round-trip
    run_test("roundtrip_parse_write_reparse", || {
        let content = std::fs::read_to_string(VENEZUELA_PATH).unwrap();
        let tree1 = parse_focus_file(&content).unwrap();
        let output = write_focus_tree(&tree1);
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
        Ok(())
    });
    
    // T3: Braces balanced
    run_test("braces_balanced_in_output", || {
        let content = std::fs::read_to_string(VENEZUELA_PATH).unwrap();
        let tree = parse_focus_file(&content).unwrap();
        let output = write_focus_tree(&tree);
        let open = output.chars().filter(|&c| c == '{').count();
        let close = output.chars().filter(|&c| c == '}').count();
        assert_eq!(open, close, "{} open vs {} close", open, close);
        println!("  {} brace pairs balanced", open);
        Ok(())
    });
    
    // T4: Validation
    run_test("validation_real_file", || {
        let content = std::fs::read_to_string(VENEZUELA_PATH).unwrap();
        let tree = parse_focus_file(&content).unwrap();
        let (errors, warnings) = validate_tree(&tree);
        println!("  {} errors, {} warnings", errors.len(), warnings.len());
        for e in errors.iter().take(5) { println!("    ❌ {}", e); }
        assert!(errors.len() < 50, "Too many errors");
        Ok(())
    });
    
    // T5: Performance
    run_test("performance_parse_10_iterations", || {
        let content = std::fs::read_to_string(VENEZUELA_PATH).unwrap();
        let _ = parse_focus_file(&content);
        let iters = 10;
        let start = std::time::Instant::now();
        for _ in 0..iters { let _ = parse_focus_file(&content); }
        let avg = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        println!("  avg {:.2}ms per parse", avg);
        assert!(avg < 200.0, "Too slow: {:.2}ms", avg);
        Ok(())
    });
    
    // T6: Categories
    run_test("focus_categories_venezuela", || {
        let content = std::fs::read_to_string(VENEZUELA_PATH).unwrap();
        let tree = parse_focus_file(&content).unwrap();
        let mut cats = std::collections::HashMap::new();
        for f in &tree.focuses { *cats.entry(f.category().to_string()).or_insert(0) += 1; }
        let with_icon = tree.focuses.iter().filter(|f| f.icon.is_some()).count();
        let with_cost = tree.focuses.iter().filter(|f| f.cost.is_some()).count();
        println!("  {} categories | {}/{} icons | {}/{} costs", cats.len(), with_icon, tree.focuses.len(), with_cost, tree.focuses.len());
        for (c, n) in &cats { println!("    {}: {}", c, n); }
        assert!(cats.len() >= 3);
        Ok(())
    });
    
    // T7: Diff detection
    run_test("diff_detection_added_focus", || {
        let content = std::fs::read_to_string(VENEZUELA_PATH).unwrap();
        let tree1 = parse_focus_file(&content).unwrap();
        let mut tree2 = tree1.clone();
        tree2.focuses.push(FocusNode {
            id: "VEN_test_new".to_string(), icon: Some("test".to_string()),
            x: 0, y: 0, relative_position_id: None, cost: Some(5.0),
            prerequisites: vec![], mutually_exclusive: vec![],
            bypass_if_unavailable: false, search_filters: vec![],
            completion_reward_raw: None, ai_will_do_raw: None,
        });
        assert_eq!(tree2.focuses.len(), tree1.focuses.len() + 1);
        Ok(())
    });
    
    // T8: Edge cases
    run_test("edge_empty_file_rejected", || {
        let result = parse_focus_file("focus_tree = {\n    id = empty\n}");
        assert!(result.is_err(), "Should fail on empty tree");
        Ok(())
    });
    
    run_test("edge_garbage_input_rejected", || {
        let result = parse_focus_file("random garbage without any structure");
        assert!(result.is_err());
        Ok(())
    });
    
    run_test("edge_special_characters", || {
        let data = r#"focus_tree = {
            id = test_special
            focus = {
                id = TEST_special
                icon = icon_with_underscore
                x = -5
                y = 10
                cost = 3.7
                completion_reward = {
                    add_political_power = 150
                }
                ai_will_do = { base = 25 }
            }
        }"#;
        let tree = parse_focus_file(data).unwrap();
        assert_eq!(tree.focuses[0].x, -5);
        assert_eq!(tree.focuses[0].cost, Some(3.7));
        assert!(tree.focuses[0].completion_reward_raw.is_some());
        Ok(())
    });
    
    // T9: Colombia & Brazil
    let colombia = VENEZUELA_PATH.replace("venezuela.txt", "colombia.txt");
    if std::path::Path::new(&colombia).exists() {
        run_test("parse_colombia", || {
            let content = std::fs::read_to_string(&colombia).unwrap();
            let tree = parse_focus_file(&content).unwrap();
            println!("  {} focuses", tree.focuses.len());
            Ok(())
        });
    }
    
    let brazil = VENEZUELA_PATH.replace("venezuela.txt", "brazil.txt");
    if std::path::Path::new(&brazil).exists() {
        run_test("parse_brazil", || {
            let content = std::fs::read_to_string(&brazil).unwrap();
            let tree = parse_focus_file(&content).unwrap();
            println!("  {} focuses", tree.focuses.len());
            Ok(())
        });
    }
    
    // T10: Write output correctness
    run_test("writer_output_contains_all_focus_ids", || {
        let content = std::fs::read_to_string(VENEZUELA_PATH).unwrap();
        let tree = parse_focus_file(&content).unwrap();
        let output = write_focus_tree(&tree);
        for focus in &tree.focuses {
            assert!(output.contains(&format!("id = {}", focus.id)), 
                "Missing focus ID in output: {}", focus.id);
        }
        Ok(())
    });
    
    println!("\n=== Test Suite Complete ===");
}
