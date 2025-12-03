// Instruction registry for Frel compiler
//
// This module defines valid instructions and their parameter keywords.
// Used during semantic analysis to validate instruction usage and
// distinguish contextual keywords from variable references.

use std::collections::HashMap;

/// Registry of all known instructions and their valid parameters
pub struct InstructionRegistry {
    instructions: HashMap<&'static str, InstructionDef>,
    /// Set of all known shorthand instruction names (no params)
    shorthands: HashMap<&'static str, ()>,
}

/// Definition of an instruction
#[derive(Debug, Clone)]
pub struct InstructionDef {
    /// Name of the instruction
    pub name: &'static str,
    /// Parameter definitions
    pub params: Vec<ParamDef>,
}

/// Definition of an instruction parameter
#[derive(Debug, Clone)]
pub struct ParamDef {
    /// Parameter name (empty string for positional/unnamed params)
    pub name: &'static str,
    /// What kind of values this parameter accepts
    pub kind: ParamKind,
}

/// The kind of values a parameter accepts
#[derive(Debug, Clone)]
pub enum ParamKind {
    /// Any expression (numeric, string, color, etc.)
    Expression,
    /// Must be one of these keyword values
    Keywords(&'static [&'static str]),
    /// Either a keyword from the list, or any expression
    KeywordOrExpr(&'static [&'static str]),
}

impl InstructionRegistry {
    /// Create a new instruction registry with all known instructions
    pub fn new() -> Self {
        let mut registry = Self {
            instructions: HashMap::new(),
            shorthands: HashMap::new(),
        };
        registry.register_all();
        registry
    }

    /// Get the definition of an instruction by name
    pub fn get(&self, name: &str) -> Option<&InstructionDef> {
        self.instructions.get(name)
    }

    /// Check if a name is a known shorthand instruction
    pub fn is_shorthand(&self, name: &str) -> bool {
        self.shorthands.contains_key(name)
    }

    /// Check if a name is a known instruction (including shorthands)
    pub fn is_known(&self, name: &str) -> bool {
        self.instructions.contains_key(name) || self.shorthands.contains_key(name)
    }

    /// Check if a value is a valid keyword for a specific instruction parameter
    pub fn is_valid_keyword(&self, instr_name: &str, param_name: &str, value: &str) -> bool {
        if let Some(instr) = self.instructions.get(instr_name) {
            for param in &instr.params {
                // Match if param names are equal, or if the registry uses "" for positional
                // params and the parser uses "value" as the default name
                if Self::params_match(param.name, param_name) {
                    return match &param.kind {
                        ParamKind::Expression => false, // No keywords, must be expression
                        ParamKind::Keywords(keywords) => keywords.contains(&value),
                        ParamKind::KeywordOrExpr(keywords) => keywords.contains(&value),
                    };
                }
            }
        }
        false
    }

    /// Get the list of valid keywords for an instruction parameter
    pub fn valid_keywords(&self, instr_name: &str, param_name: &str) -> Option<&'static [&'static str]> {
        if let Some(instr) = self.instructions.get(instr_name) {
            for param in &instr.params {
                if Self::params_match(param.name, param_name) {
                    return match &param.kind {
                        ParamKind::Expression => None,
                        ParamKind::Keywords(keywords) => Some(keywords),
                        ParamKind::KeywordOrExpr(keywords) => Some(keywords),
                    };
                }
            }
        }
        None
    }

    /// Check if an instruction parameter accepts expressions (not just keywords)
    pub fn accepts_expression(&self, instr_name: &str, param_name: &str) -> bool {
        if let Some(instr) = self.instructions.get(instr_name) {
            for param in &instr.params {
                if Self::params_match(param.name, param_name) {
                    return matches!(param.kind, ParamKind::Expression | ParamKind::KeywordOrExpr(_));
                }
            }
        }
        // Unknown instruction/param - default to accepting expressions
        true
    }

    /// Check if parameter names match.
    /// The registry uses "" for positional params, but the parser uses "value" as the default name.
    fn params_match(registry_name: &str, parsed_name: &str) -> bool {
        registry_name == parsed_name
            || (registry_name.is_empty() && parsed_name == "value")
            || (registry_name == "value" && parsed_name.is_empty())
    }

    fn register_all(&mut self) {
        // Dimension instructions
        self.register_instruction("width", vec![
            ParamDef { name: "", kind: ParamKind::KeywordOrExpr(&["expand", "container", "content"]) },
            ParamDef { name: "value", kind: ParamKind::KeywordOrExpr(&["expand", "container", "content"]) },
            ParamDef { name: "min", kind: ParamKind::Expression },
            ParamDef { name: "max", kind: ParamKind::Expression },
        ]);
        self.register_instruction("height", vec![
            ParamDef { name: "", kind: ParamKind::KeywordOrExpr(&["expand", "container", "content"]) },
            ParamDef { name: "value", kind: ParamKind::KeywordOrExpr(&["expand", "container", "content"]) },
            ParamDef { name: "min", kind: ParamKind::Expression },
            ParamDef { name: "max", kind: ParamKind::Expression },
        ]);
        self.register_instruction("size", vec![
            ParamDef { name: "", kind: ParamKind::Expression },
        ]);

        // Dimension shorthands
        self.register_shorthand("fit_content");
        self.register_shorthand("fill_width");
        self.register_shorthand("fill_height");
        self.register_shorthand("fill");
        self.register_shorthand("expand");

        // Position
        self.register_instruction("position", vec![
            ParamDef { name: "top", kind: ParamKind::Expression },
            ParamDef { name: "left", kind: ParamKind::Expression },
            ParamDef { name: "right", kind: ParamKind::Expression },
            ParamDef { name: "bottom", kind: ParamKind::Expression },
        ]);

        // Surrounding (padding, border, margin)
        for instr in &["padding", "margin"] {
            self.register_instruction(instr, vec![
                ParamDef { name: "", kind: ParamKind::Expression },
                ParamDef { name: "top", kind: ParamKind::Expression },
                ParamDef { name: "right", kind: ParamKind::Expression },
                ParamDef { name: "bottom", kind: ParamKind::Expression },
                ParamDef { name: "left", kind: ParamKind::Expression },
                ParamDef { name: "horizontal", kind: ParamKind::Expression },
                ParamDef { name: "vertical", kind: ParamKind::Expression },
            ]);
        }
        self.register_instruction("border", vec![
            ParamDef { name: "", kind: ParamKind::Expression },
            ParamDef { name: "top", kind: ParamKind::Expression },
            ParamDef { name: "right", kind: ParamKind::Expression },
            ParamDef { name: "bottom", kind: ParamKind::Expression },
            ParamDef { name: "left", kind: ParamKind::Expression },
            ParamDef { name: "horizontal", kind: ParamKind::Expression },
            ParamDef { name: "vertical", kind: ParamKind::Expression },
            ParamDef { name: "color", kind: ParamKind::Expression },
            ParamDef { name: "width", kind: ParamKind::Expression },
        ]);

        // Fill strategy
        self.register_instruction("fill_strategy", vec![
            ParamDef { name: "", kind: ParamKind::Keywords(&["constrain", "constrain_reverse", "resize_to_max"]) },
        ]);
        self.register_shorthand("constrain");
        self.register_shorthand("constrain_reverse");
        self.register_shorthand("resize_to_max");

        // Gap
        self.register_instruction("gap", vec![
            ParamDef { name: "", kind: ParamKind::Expression },
            ParamDef { name: "width", kind: ParamKind::Expression },
            ParamDef { name: "height", kind: ParamKind::Expression },
        ]);

        // Alignment
        self.register_instruction("align_self", vec![
            ParamDef { name: "horizontal", kind: ParamKind::Keywords(&["start", "center", "end"]) },
            ParamDef { name: "vertical", kind: ParamKind::Keywords(&["top", "center", "baseline", "bottom"]) },
        ]);
        self.register_instruction("align_items", vec![
            ParamDef { name: "horizontal", kind: ParamKind::Keywords(&["start", "center", "end"]) },
            ParamDef { name: "vertical", kind: ParamKind::Keywords(&["top", "center", "baseline", "bottom"]) },
        ]);
        self.register_instruction("align_relative", vec![
            ParamDef { name: "horizontal", kind: ParamKind::Keywords(&["before", "start", "center", "end", "after"]) },
            ParamDef { name: "vertical", kind: ParamKind::Keywords(&["above", "start", "center", "end", "below"]) },
        ]);

        // Alignment shorthands
        self.register_shorthand("align_self_center");
        self.register_shorthand("align_items_center");
        // The documentation shows patterns like align_self_start_top, align_items_center_bottom, etc.
        // Register common combinations (explicitly listed for 'static lifetime)
        self.register_shorthand("align_self_start_top");
        self.register_shorthand("align_self_start_center");
        self.register_shorthand("align_self_start_bottom");
        self.register_shorthand("align_self_start_baseline");
        self.register_shorthand("align_self_center_top");
        self.register_shorthand("align_self_center_center");
        self.register_shorthand("align_self_center_bottom");
        self.register_shorthand("align_self_center_baseline");
        self.register_shorthand("align_self_end_top");
        self.register_shorthand("align_self_end_center");
        self.register_shorthand("align_self_end_bottom");
        self.register_shorthand("align_self_end_baseline");
        self.register_shorthand("align_items_start_top");
        self.register_shorthand("align_items_start_center");
        self.register_shorthand("align_items_start_bottom");
        self.register_shorthand("align_items_start_baseline");
        self.register_shorthand("align_items_center_top");
        self.register_shorthand("align_items_center_center");
        self.register_shorthand("align_items_center_bottom");
        self.register_shorthand("align_items_center_baseline");
        self.register_shorthand("align_items_end_top");
        self.register_shorthand("align_items_end_center");
        self.register_shorthand("align_items_end_bottom");
        self.register_shorthand("align_items_end_baseline");

        // Spacing
        self.register_shorthand("space_around");
        self.register_shorthand("space_between");

        // Scroll
        self.register_instruction("scroll", vec![
            ParamDef { name: "", kind: ParamKind::Keywords(&["horizontal", "vertical", "both"]) },
        ]);

        // Background
        self.register_instruction("background", vec![
            ParamDef { name: "color", kind: ParamKind::Expression },
            ParamDef { name: "opacity", kind: ParamKind::Expression },
            ParamDef { name: "gradient", kind: ParamKind::Expression },
            ParamDef { name: "image", kind: ParamKind::Expression },
        ]);

        // Corner radius
        self.register_instruction("corner_radius", vec![
            ParamDef { name: "", kind: ParamKind::Expression },
            ParamDef { name: "top", kind: ParamKind::Expression },
            ParamDef { name: "bottom", kind: ParamKind::Expression },
            ParamDef { name: "left", kind: ParamKind::Expression },
            ParamDef { name: "right", kind: ParamKind::Expression },
            ParamDef { name: "top_left", kind: ParamKind::Expression },
            ParamDef { name: "top_right", kind: ParamKind::Expression },
            ParamDef { name: "bottom_left", kind: ParamKind::Expression },
            ParamDef { name: "bottom_right", kind: ParamKind::Expression },
        ]);

        // Shadow
        self.register_instruction("shadow", vec![
            ParamDef { name: "color", kind: ParamKind::Expression },
            ParamDef { name: "offset_x", kind: ParamKind::Expression },
            ParamDef { name: "offset_y", kind: ParamKind::Expression },
            ParamDef { name: "blur", kind: ParamKind::Expression },
        ]);

        // Cursor
        self.register_instruction("cursor", vec![
            ParamDef { name: "", kind: ParamKind::Keywords(&[
                "default", "pointer", "text", "crosshair", "move", "none", "grab", "grabbing"
            ]) },
        ]);

        // Tint
        self.register_instruction("tint", vec![
            ParamDef { name: "", kind: ParamKind::Expression },
        ]);

        // Focusable
        self.register_instruction("focusable", vec![
            ParamDef { name: "", kind: ParamKind::KeywordOrExpr(&["false", "programmatic"]) },
            ParamDef { name: "order", kind: ParamKind::KeywordOrExpr(&["false", "programmatic"]) },
        ]);
        self.register_shorthand("focusable");
        self.register_shorthand("not_focusable");
        self.register_shorthand("autofocus");
        self.register_shorthand("focus_trap");

        // Pointer events
        self.register_instruction("pointer_events", vec![
            ParamDef { name: "", kind: ParamKind::Keywords(&["enabled", "disabled"]) },
        ]);
        self.register_shorthand("with_pointer_events");
        self.register_shorthand("no_pointer_events");

        // Text
        self.register_shorthand("no_select");
        self.register_shorthand("underline");
        self.register_shorthand("small_caps");

        // Font
        self.register_instruction("font", vec![
            ParamDef { name: "name", kind: ParamKind::Expression },
            ParamDef { name: "size", kind: ParamKind::Expression },
            ParamDef { name: "weight", kind: ParamKind::Expression },
            ParamDef { name: "color", kind: ParamKind::Expression },
        ]);

        // Line height
        self.register_instruction("line_height", vec![
            ParamDef { name: "", kind: ParamKind::Expression },
            ParamDef { name: "height", kind: ParamKind::Expression },
        ]);

        // Text wrapping
        self.register_instruction("text_wrap", vec![
            ParamDef { name: "", kind: ParamKind::Keywords(&["none", "wrap"]) },
        ]);

        // Text overflow
        self.register_instruction("text_overflow", vec![
            ParamDef { name: "", kind: ParamKind::Keywords(&["visible", "ellipsis"]) },
        ]);

        // Letter spacing
        self.register_instruction("letter_spacing", vec![
            ParamDef { name: "", kind: ParamKind::Expression },
            ParamDef { name: "value", kind: ParamKind::Expression },
        ]);

        // Stereotype
        self.register_instruction("stereotype", vec![
            ParamDef { name: "", kind: ParamKind::Keywords(&["cancel", "save"]) },
        ]);

        // Event handlers - these take closures, not keyword params
        for event in &[
            "on_click", "on_double_click", "on_long_press",
            "on_right_click", "on_context_menu",
            "on_hover_start", "on_hover_end",
            "on_key_down", "on_key_up", "on_key_press",
            "on_scroll", "on_resize",
            "on_focus", "on_blur",
            "on_drag_start", "on_drag_end", "on_drag_enter", "on_drag_leave", "on_drop",
        ] {
            self.register_instruction(event, vec![
                ParamDef { name: "", kind: ParamKind::Expression },
            ]);
        }
    }

    fn register_instruction(&mut self, name: &'static str, params: Vec<ParamDef>) {
        self.instructions.insert(name, InstructionDef { name, params });
    }

    fn register_shorthand(&mut self, name: &'static str) {
        self.shorthands.insert(name, ());
    }
}

impl Default for InstructionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Global singleton for the instruction registry
use std::sync::OnceLock;

static INSTRUCTION_REGISTRY: OnceLock<InstructionRegistry> = OnceLock::new();

/// Get the global instruction registry instance
pub fn instruction_registry() -> &'static InstructionRegistry {
    INSTRUCTION_REGISTRY.get_or_init(InstructionRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = InstructionRegistry::new();
        assert!(registry.is_known("cursor"));
        assert!(registry.is_known("align_items"));
        assert!(registry.is_known("focusable"));
        assert!(registry.is_shorthand("constrain"));
    }

    #[test]
    fn test_cursor_keywords() {
        let registry = InstructionRegistry::new();
        assert!(registry.is_valid_keyword("cursor", "", "pointer"));
        assert!(registry.is_valid_keyword("cursor", "", "default"));
        assert!(!registry.is_valid_keyword("cursor", "", "invalid_cursor"));
    }

    #[test]
    fn test_align_items_keywords() {
        let registry = InstructionRegistry::new();
        assert!(registry.is_valid_keyword("align_items", "horizontal", "center"));
        assert!(registry.is_valid_keyword("align_items", "horizontal", "start"));
        assert!(registry.is_valid_keyword("align_items", "vertical", "top"));
        assert!(!registry.is_valid_keyword("align_items", "horizontal", "top")); // top is for vertical
    }

    #[test]
    fn test_width_accepts_expressions() {
        let registry = InstructionRegistry::new();
        // width accepts both keywords and expressions
        assert!(registry.accepts_expression("width", ""));
        assert!(registry.is_valid_keyword("width", "", "expand"));
    }

    #[test]
    fn test_valid_keywords_lookup() {
        let registry = InstructionRegistry::new();
        let keywords = registry.valid_keywords("cursor", "");
        assert!(keywords.is_some());
        assert!(keywords.unwrap().contains(&"pointer"));
    }
}
