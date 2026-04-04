/// Writer to serialize FocusTree back to Paradox HOI4 format

use std::fmt::Write;

use crate::model::{FocusNode, FocusTree};

/// Serialize a focus tree to Paradox text format
pub fn write_focus_tree(tree: &FocusTree) -> String {
    let mut out = String::with_capacity(128 * 1024);
    
    writeln!(out, "focus_tree = {{").unwrap();
    writeln!(out, "\tid = {}", tree.id).unwrap();
    writeln!(out).unwrap();
    
    // Write shortcuts
    for shortcut in &tree.shortcuts {
        writeln!(out, "\tshortcut = {{").unwrap();
        writeln!(out, "\t\tname = {}", shortcut.name).unwrap();
        writeln!(out, "\t\ttarget = {}", shortcut.target).unwrap();
        writeln!(out, "\t\tscroll_wheel_factor = 0.80").unwrap();
        writeln!(out, "\t}}").unwrap();
    }
    if !tree.shortcuts.is_empty() {
        writeln!(out).unwrap();
    }
    
    // Write each focus
    for (i, focus) in tree.focuses.iter().enumerate() {
        if i > 0 {
            writeln!(out).unwrap();
        }
        write_focus_node(&mut out, focus);
    }
    
    writeln!(out, "}}").unwrap();
    out
}

fn write_focus_node(out: &mut String, focus: &FocusNode) {
    writeln!(out, "\tfocus = {{").unwrap();
    writeln!(out, "\t\tid = {}", focus.id).unwrap();
    
    if let Some(ref icon) = focus.icon {
        writeln!(out, "\t\ticon = {}", icon).unwrap();
    }
    
    writeln!(out, "\t\tx = {}", focus.x).unwrap();
    writeln!(out, "\t\ty = {}", focus.y).unwrap();
    
    if let Some(ref rel_id) = focus.relative_position_id {
        writeln!(out, "\t\trelative_position_id = {}", rel_id).unwrap();
    }
    
    if let Some(cost) = focus.cost {
        if cost.fract() == 0.0 {
            writeln!(out, "\t\tcost = {}", cost as i32).unwrap();
        } else {
            writeln!(out, "\t\tcost = {}", cost).unwrap();
        }
    }
    
    // Prerequisites
    for prereq in &focus.prerequisites {
        writeln!(out, "\t\tprerequisite = {{ focus = {} }}", prereq).unwrap();
    }
    
    // Mutually exclusive
    for me in &focus.mutually_exclusive {
        writeln!(out, "\t\tmutually_exclusive = {{ focus = {} }}", me).unwrap();
    }
    
    if focus.bypass_if_unavailable {
        writeln!(out, "\t\tbypass_if_unavailable = yes").unwrap();
    }
    
    // Search filters
    if !focus.search_filters.is_empty() {
        if focus.search_filters.len() == 1 {
            writeln!(out, "\t\tsearch_filters = {{ {} }}", focus.search_filters[0]).unwrap();
        } else {
            write!(out, "\t\tsearch_filters = {{").unwrap();
            for filter in &focus.search_filters {
                write!(out, " {}", filter).unwrap();
            }
            writeln!(out, " }}").unwrap();
        }
    }
    
    // Write complex blocks: strip outer braces from stored text, re-write with proper format
    write_block(out, "available", &focus.available_raw);
    write_block(out, "bypass", &focus.bypass_raw);
    write_block(out, "completion_reward", &focus.completion_reward_raw);
    write_block(out, "immediate", &focus.immediate_raw);
    write_block(out, "ai_will_do", &focus.ai_will_do_raw);
    
    writeln!(out, "\t}}").unwrap();
}

/// Write a complex block, stripping outer braces from stored text
fn write_block(out: &mut String, key: &str, raw: &Option<String>) {
    if let Some(ref content) = raw {
        // Strip outer braces if present: "{\n...\n}" -> inner content
        let trimmed = content.trim();
        let inner = if trimmed.starts_with('{') && trimmed.ends_with('}') {
            // Remove first { and last }
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };
        
        writeln!(out, "\t\t{} = {{", key).unwrap();
        // Write inner content with extra tab indent
        for line in inner.lines() {
            writeln!(out, "\t\t\t{}", line.trim()).unwrap();
        }
        writeln!(out, "\t\t}}").unwrap();
    }
}

/// Write a single focus node to string (for preview)
#[allow(dead_code)]
pub fn write_single_focus(focus: &FocusNode) -> String {
    let mut out = String::new();
    write_focus_node(&mut out, focus);
    out
}

/// Generate a diff between two versions of a tree
#[allow(dead_code)]
pub fn generate_diff(old_tree: &FocusTree, new_tree: &FocusTree) -> String {
    let mut out = String::new();
    
    let old_ids: std::collections::HashSet<&str> = old_tree.focuses.iter().map(|f| f.id.as_str()).collect();
    let new_ids: std::collections::HashSet<&str> = new_tree.focuses.iter().map(|f| f.id.as_str()).collect();
    
    let added: Vec<_> = new_tree.focuses.iter().filter(|f| !old_ids.contains(f.id.as_str())).collect();
    if !added.is_empty() {
        writeln!(out, "➕ Added {} focus(es):", added.len()).unwrap();
        for f in &added { writeln!(out, "  + {}", f.id).unwrap(); }
        writeln!(out).unwrap();
    }
    
    let removed: Vec<_> = old_tree.focuses.iter().filter(|f| !new_ids.contains(f.id.as_str())).collect();
    if !removed.is_empty() {
        writeln!(out, "➖ Removed {} focus(es):", removed.len()).unwrap();
        for f in &removed { writeln!(out, "  - {}", f.id).unwrap(); }
        writeln!(out).unwrap();
    }
    
    for new_focus in &new_tree.focuses {
        if let Some(old_focus) = old_tree.focuses.iter().find(|f| f.id == new_focus.id) {
            let mut changes = Vec::new();
            if old_focus.icon != new_focus.icon { changes.push(format!("icon changed")); }
            if old_focus.x != new_focus.x || old_focus.y != new_focus.y { changes.push(format!("pos changed")); }
            if old_focus.cost != new_focus.cost { changes.push(format!("cost changed")); }
            if old_focus.prerequisites != new_focus.prerequisites { changes.push("prerequisites changed".to_string()); }
            if old_focus.completion_reward_raw != new_focus.completion_reward_raw { changes.push("completion_reward changed".to_string()); }
            
            if !changes.is_empty() {
                writeln!(out, "✏️ Modified: {}", new_focus.id).unwrap();
                for c in &changes { writeln!(out, "   • {}", c).unwrap(); }
            }
        }
    }
    
    if added.is_empty() && removed.is_empty() && out.is_empty() {
        out.push_str("No changes detected");
    }
    
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FocusNode;
    
    #[test]
    fn test_write_focus() {
        let focus = FocusNode {
            id: "TEST_example".to_string(),
            icon: Some("generic_industry".to_string()),
            x: 5, y: 2,
            relative_position_id: Some("TEST_parent".to_string()),
            cost: Some(5.0),
            prerequisites: vec!["TEST_parent".to_string()],
            mutually_exclusive: vec![],
            bypass_if_unavailable: false,
            available_raw: None,
            completion_reward_raw: None,
            immediate_raw: None,
            ai_will_do_raw: Some("{\n\t\t\tbase = 10\n\t\t}".to_string()),
            search_filters: vec!["FOCUS_FILTER_INDUSTRY".to_string()],
            bypass_raw: None,
        };
        
        let output = write_single_focus(&focus);
        assert!(output.contains("id = TEST_example"));
        assert!(output.contains("icon = generic_industry"));
        assert!(output.contains("x = 5"));
        assert!(output.contains("prerequisite = { focus = TEST_parent }"));
    }
    
    #[test]
    fn test_diff_generation() {
        let old = FocusTree {
            id: "test".to_string(), shortcuts: vec![], modified: false,
            focuses: vec![
                FocusNode { id: "A".to_string(), icon: None, x: 0, y: 0, relative_position_id: None, cost: Some(5.0), prerequisites: vec![], mutually_exclusive: vec![], bypass_if_unavailable: false, available_raw: None, completion_reward_raw: None, immediate_raw: None, ai_will_do_raw: None, search_filters: vec![], bypass_raw: None },
            ],
        };
        let new = FocusTree {
            id: "test".to_string(), shortcuts: vec![], modified: false,
            focuses: vec![
                FocusNode { id: "A".to_string(), icon: Some("new_icon".to_string()), x: 5, y: 0, relative_position_id: None, cost: Some(5.0), prerequisites: vec![], mutually_exclusive: vec![], bypass_if_unavailable: false, available_raw: None, completion_reward_raw: None, immediate_raw: None, ai_will_do_raw: None, search_filters: vec![], bypass_raw: None },
                FocusNode { id: "B".to_string(), icon: None, x: 0, y: 0, relative_position_id: None, cost: Some(5.0), prerequisites: vec![], mutually_exclusive: vec![], bypass_if_unavailable: false, available_raw: None, completion_reward_raw: None, immediate_raw: None, ai_will_do_raw: None, search_filters: vec![], bypass_raw: None },
            ],
        };
        let diff = generate_diff(&old, &new);
        assert!(diff.contains("Added"));
        assert!(diff.contains("Modified"));
    }
}
