/// Data model for HOI4 focus trees

use serde::{Deserialize, Serialize};

/// Complete focus tree (one file like venezuela.txt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTree {
    pub id: String,
    pub shortcuts: Vec<Shortcut>,
    pub focuses: Vec<FocusNode>,
    /// Whether the tree has been modified
    pub modified: bool,
}

/// Quick reference to a root focus (shown in sidebar)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub name: String,
    pub target: String,
}

/// A single focus node in the tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusNode {
    /// Unique identifier: VEN_something
    pub id: String,
    
    /// GFX icon name
    pub icon: Option<String>,
    
    /// X position in the tree (grid units, 96px each)
    pub x: i32,
    
    /// Y position in the tree (grid units, 130px each)
    pub y: i32,
    
    /// If set, x/y are relative to this focus
    pub relative_position_id: Option<String>,
    
    /// Cost in days (typically 3.7 - 10)
    pub cost: Option<f32>,
    
    /// Prerequisites: other focus IDs that must be completed first
    pub prerequisites: Vec<String>,
    
    /// Mutually exclusive focuses: other focus IDs
    pub mutually_exclusive: Vec<String>,
    
    /// If true, focus can be bypassed even if unavailable
    pub bypass_if_unavailable: bool,
    
    /// Availability trigger (raw Paradox text)
    pub available_raw: Option<String>,
    
    /// Effects on completion (raw Paradox text)
    pub completion_reward_raw: Option<String>,
    
    /// Immediate effects (raw Paradox text)
    pub immediate_raw: Option<String>,
    
    /// AI priority weights (raw Paradox text)
    pub ai_will_do_raw: Option<String>,
    
    /// Search filters for in-game categorization
    pub search_filters: Vec<String>,
    
    /// Whether this focus has a bypass defined
    pub bypass_raw: Option<String>,
}

impl FocusNode {
    /// Get display name (id without TAG_ prefix)
    pub fn display_name(&self) -> &str {
        self.id.strip_prefix("VEN_").unwrap_or(&self.id)
    }
    
    /// Get category based on search filters
    pub fn category(&self) -> &str {
        for filter in &self.search_filters {
            if filter.contains("INDUSTRY") || filter.contains("ECONOMY") {
                return "Industry";
            }
            if filter.contains("MILITARY") || filter.contains("ARMY") 
               || filter.contains("NAVY") || filter.contains("AIR") {
                return "Military";
            }
            if filter.contains("POLITICAL") {
                return "Political";
            }
            if filter.contains("RESEARCH") {
                return "Research";
            }
            if filter.contains("FOREIGN") {
                return "Foreign";
            }
        }
        "Other"
    }
    
    /// Get category icon
    pub fn category_icon(&self) -> &str {
        match self.category() {
            "Industry" => "🏭",
            "Military" => "⚔️",
            "Political" => "🏛️",
            "Research" => "🔬",
            "Foreign" => "🌍",
            _ => "📋",
        }
    }
    
    /// Get color for category
    pub fn category_color(&self) -> [f32; 3] {
        match self.category() {
            "Industry" => [0.9, 0.7, 0.2],   // Gold
            "Military" => [0.9, 0.3, 0.3],    // Red
            "Political" => [0.3, 0.6, 1.0],   // Blue
            "Research" => [0.3, 0.9, 0.5],    // Green
            "Foreign" => [0.8, 0.5, 0.9],     // Purple
            _ => [0.7, 0.7, 0.7],             // Gray
        }
    }
    
    /// Compute absolute screen position given pixel grid size
    pub fn pixel_position(&self, tree: &FocusTree, grid_w: f32, grid_h: f32) -> (f32, f32) {
        if let Some(ref parent_id) = self.relative_position_id {
            if let Some(parent) = tree.focuses.iter().find(|f| f.id == *parent_id) {
                let (px, py) = parent.pixel_position(tree, grid_w, grid_h);
                return (px + self.x as f32 * grid_w, py + self.y as f32 * grid_h);
            }
        }
        // Base positions use absolute coordinates
        (self.x as f32 * grid_w, self.y as f32 * grid_h)
    }
}

/// Validation error for a focus
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Focus ID doesn't have the expected TAG_ prefix
    InvalidIdFormat(String),
    /// A prerequisite focus doesn't exist
    MissingPrerequisite(String),
    /// A mutually exclusive focus doesn't exist
    MissingMutuallyExclusive(String),
    /// Duplicate focus ID found
    DuplicateId(String),
    /// Cost is outside normal range
    UnusualCost { cost: f32, min: f32, max: f32 },
    /// Position is outside reasonable bounds
    PositionOutOfBounds { x: i32, y: i32 },
    /// Focus has completion_reward but no ai_will_do
    MissingAiWillDo(String),
    /// General warning (non-blocking)
    Warning(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidIdFormat(id) => write!(f, "Invalid ID format: '{}' should contain '_'", id),
            ValidationError::MissingPrerequisite(id) => write!(f, "Missing prerequisite: '{}'", id),
            ValidationError::MissingMutuallyExclusive(id) => write!(f, "Missing mutually exclusive: '{}'", id),
            ValidationError::DuplicateId(id) => write!(f, "Duplicate ID: '{}'", id),
            ValidationError::UnusualCost { cost, min, max } => write!(f, "Unusual cost {} (normal {}-{})", cost, min, max),
            ValidationError::PositionOutOfBounds { x, y } => write!(f, "Position ({}, {}) out of bounds", x, y),
            ValidationError::MissingAiWillDo(id) => write!(f, "Focus '{}' missing ai_will_do", id),
            ValidationError::Warning(msg) => write!(f, "Warning: {}", msg),
        }
    }
}

/// Validation result
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

/// Undo/redo history entry
#[derive(Clone)]
pub struct HistoryEntry {
    pub description: String,
    pub tree_snapshot: String, // JSON snapshot
}
