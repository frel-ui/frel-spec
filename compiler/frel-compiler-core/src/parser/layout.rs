// Layout Mini-DSL Parser for Frel
//
// This module parses the content inside """layout ... """ blocks.
// The layout syntax is a mini-DSL with:
// - Instructions (lines starting with `..`)
// - Column size lines (size tokens without `|`)
// - Grid row lines (`| cell | cell |` with optional row size)
// - Cell content (slot name, alignment modifiers, merge indicators)

use crate::ast::{
    HAlign, InstructionExpr, LayoutCell, LayoutRow, LayoutSize, LayoutStmt, MergeDirection, VAlign,
};
use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::source::Span;

/// Layout parser state
pub struct LayoutParser<'a> {
    /// The raw layout content (between """layout and """)
    content: &'a str,
    /// Lines of content
    lines: Vec<&'a str>,
    /// Current line index
    current_line: usize,
    /// Base span offset (start of layout block in original source)
    base_offset: u32,
    /// Accumulated diagnostics
    diagnostics: &'a mut Diagnostics,
}

/// Classification of a line in the layout content
#[derive(Debug, PartialEq)]
enum LineType {
    /// Empty or whitespace-only line
    Empty,
    /// Instruction line starting with `..`
    Instruction,
    /// Column sizes line (contains size tokens but no `|`)
    ColumnSizes,
    /// Grid row line (contains `|` characters)
    GridRow,
}

impl<'a> LayoutParser<'a> {
    /// Create a new layout parser
    pub fn new(content: &'a str, base_offset: u32, diagnostics: &'a mut Diagnostics) -> Self {
        let lines: Vec<&str> = content.lines().collect();
        Self {
            content,
            lines,
            current_line: 0,
            base_offset,
            diagnostics,
        }
    }

    /// Parse the layout content and return a LayoutStmt
    pub fn parse(&mut self) -> Option<LayoutStmt> {
        let mut instructions = Vec::new();
        let mut column_sizes = Vec::new();
        let mut rows = Vec::new();

        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line];
            let line_type = self.classify_line(line);

            match line_type {
                LineType::Empty => {
                    self.current_line += 1;
                }
                LineType::Instruction => {
                    if let Some(instr) = self.parse_instruction_line(line) {
                        instructions.push(instr);
                    }
                    self.current_line += 1;
                }
                LineType::ColumnSizes => {
                    if column_sizes.is_empty() {
                        column_sizes = self.parse_column_sizes(line);
                    } else {
                        // Multiple column size lines - emit warning and use first
                        self.diagnostics.add(
                            Diagnostic::warning(
                                "multiple column size lines, using the first one",
                                self.line_span(self.current_line),
                            )
                            .with_code("E0303"),
                        );
                    }
                    self.current_line += 1;
                }
                LineType::GridRow => {
                    if let Some(row) = self.parse_grid_row(line) {
                        rows.push(row);
                    }
                    self.current_line += 1;
                }
            }
        }

        // Validate column counts
        if !rows.is_empty() {
            let expected_cols = rows[0].cells.len();
            for (i, row) in rows.iter().enumerate() {
                if row.cells.len() != expected_cols {
                    self.diagnostics.add(
                        Diagnostic::error(
                            format!(
                                "row {} has {} cells, expected {} (matching first row)",
                                i + 1,
                                row.cells.len(),
                                expected_cols
                            ),
                            self.line_span(i),
                        )
                        .with_code("E0301"),
                    );
                }
            }

            // Validate column sizes count if specified
            if !column_sizes.is_empty() && column_sizes.len() != expected_cols {
                self.diagnostics.add(
                    Diagnostic::error(
                        format!(
                            "column sizes count ({}) doesn't match column count ({})",
                            column_sizes.len(),
                            expected_cols
                        ),
                        Span::new(self.base_offset, self.base_offset + self.content.len() as u32),
                    )
                    .with_code("E0301"),
                );
            }
        }

        Some(LayoutStmt {
            instructions,
            column_sizes,
            rows,
        })
    }

    /// Classify a line by its content
    fn classify_line(&self, line: &str) -> LineType {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return LineType::Empty;
        }

        // Check for instruction prefix
        if trimmed.starts_with("..") {
            return LineType::Instruction;
        }

        // Check for grid row (has pipe characters)
        if line.contains('|') {
            return LineType::GridRow;
        }

        // Check for size tokens (numbers, ~number, #)
        if self.looks_like_sizes(trimmed) {
            return LineType::ColumnSizes;
        }

        LineType::Empty
    }

    /// Check if a line looks like column sizes
    fn looks_like_sizes(&self, line: &str) -> bool {
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                ' ' | '\t' => continue,
                '~' | '#' => return true,
                '0'..='9' => return true,
                _ => return false,
            }
        }

        false
    }

    /// Parse an instruction line (starting with `..`)
    fn parse_instruction_line(&mut self, line: &str) -> Option<InstructionExpr> {
        let trimmed = line.trim();

        // Strip the `..` prefix
        let instr_text = trimmed.strip_prefix("..")?.trim();

        // Re-lex and re-parse using the main parser
        let mut parser = super::Parser::new(instr_text);
        let result = parser.parse_instruction_expr();

        // Merge diagnostics (with span adjustment would be ideal, but for now just copy)
        // Note: In a production implementation, we'd adjust spans by self.base_offset + line offset
        for diag in parser.diagnostics.iter() {
            self.diagnostics.add(diag.clone());
        }

        result
    }

    /// Parse a column sizes line
    fn parse_column_sizes(&self, line: &str) -> Vec<LayoutSize> {
        let mut sizes = Vec::new();
        let mut chars = line.chars().peekable();
        let mut current = String::new();
        let mut in_weight = false;

        while let Some(ch) = chars.next() {
            match ch {
                ' ' | '\t' => {
                    if !current.is_empty() {
                        if let Some(size) = self.parse_size_token(&current, in_weight) {
                            sizes.push(size);
                        }
                        current.clear();
                        in_weight = false;
                    }
                }
                '~' => {
                    in_weight = true;
                }
                '#' => {
                    sizes.push(LayoutSize::Content);
                }
                '0'..='9' | '.' => {
                    current.push(ch);
                }
                _ => {
                    // Ignore other characters
                }
            }
        }

        // Handle last token
        if !current.is_empty() {
            if let Some(size) = self.parse_size_token(&current, in_weight) {
                sizes.push(size);
            }
        }

        sizes
    }

    /// Parse a size token
    fn parse_size_token(&self, token: &str, is_weight: bool) -> Option<LayoutSize> {
        if is_weight {
            token.parse::<f64>().ok().map(LayoutSize::Weight)
        } else {
            token.parse::<i32>().ok().map(LayoutSize::Fixed)
        }
    }

    /// Parse a grid row line
    fn parse_grid_row(&mut self, line: &str) -> Option<LayoutRow> {
        // Find the first pipe character
        let first_pipe = line.find('|')?;

        // Everything before the first | is the row size (optional)
        let size_part = &line[..first_pipe];
        let size = self.parse_optional_row_size(size_part.trim());

        // Split by | and parse cells
        let cells_part = &line[first_pipe..];
        let cells = self.parse_cells(cells_part);

        Some(LayoutRow { size, cells })
    }

    /// Parse optional row size from the part before first |
    fn parse_optional_row_size(&self, s: &str) -> Option<LayoutSize> {
        if s.is_empty() {
            return None;
        }

        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Check for weight (~number)
        if let Some(rest) = trimmed.strip_prefix('~') {
            return rest.trim().parse::<f64>().ok().map(LayoutSize::Weight);
        }

        // Check for content (#)
        if trimmed == "#" {
            return Some(LayoutSize::Content);
        }

        // Fixed size (number)
        trimmed.parse::<i32>().ok().map(LayoutSize::Fixed)
    }

    /// Parse cells from `| cell | cell | ... |`
    fn parse_cells(&self, s: &str) -> Vec<LayoutCell> {
        let mut cells = Vec::new();

        // Split by | and skip empty parts at start/end
        let parts: Vec<&str> = s.split('|').collect();

        // Skip first empty part (before first |) and last empty part (after last |)
        for part in &parts[1..] {
            // Check if this is the last part (after final |)
            if part.trim().is_empty() && parts.last() == Some(part) {
                continue;
            }
            cells.push(self.parse_cell(part));
        }

        cells
    }

    /// Parse a single cell content
    fn parse_cell(&self, content: &str) -> LayoutCell {
        let trimmed = content.trim();

        if trimmed.is_empty() {
            return LayoutCell {
                slot_name: None,
                h_align: HAlign::default(),
                v_align: VAlign::default(),
                merge: None,
            };
        }

        let mut h_align = HAlign::default();
        let mut v_align = VAlign::default();
        let mut merge = None;
        let mut slot_name = None;

        let mut chars = trimmed.chars().peekable();
        let mut slot_chars = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                // Check for merge indicators first (multi-character)
                '<' => {
                    if chars.next_if_eq(&'-').is_some() && chars.next_if_eq(&'-').is_some() {
                        merge = Some(MergeDirection::Left);
                    } else {
                        h_align = HAlign::Left;
                    }
                }
                '>' => {
                    // Check for --> merge
                    let mut temp_chars = chars.clone();
                    if temp_chars.next() == Some('-') && temp_chars.next() == Some('-') {
                        // It's -->
                        chars.next(); // -
                        chars.next(); // -
                        merge = Some(MergeDirection::Right);
                    } else {
                        h_align = HAlign::Right;
                    }
                }
                '^' => {
                    // Check for ^-- merge
                    let mut temp_chars = chars.clone();
                    if temp_chars.next() == Some('-') && temp_chars.next() == Some('-') {
                        chars.next(); // -
                        chars.next(); // -
                        merge = Some(MergeDirection::Up);
                    } else {
                        v_align = VAlign::Top;
                    }
                }
                '.' => {
                    v_align = VAlign::Bottom;
                }
                '!' => {
                    h_align = HAlign::Center;
                }
                '=' => {
                    v_align = VAlign::Center;
                }
                '_' => {
                    // Always baseline alignment - slot names cannot start with _
                    v_align = VAlign::Baseline;
                }
                ' ' => {
                    // Space - if we have slot chars, we're done with the identifier
                    if !slot_chars.is_empty() {
                        // Continue to allow modifiers after slot name
                    }
                }
                'v' => {
                    // Check for v-- merge (down)
                    let mut temp_chars = chars.clone();
                    if temp_chars.next() == Some('-') && temp_chars.next() == Some('-') {
                        chars.next(); // -
                        chars.next(); // -
                        merge = Some(MergeDirection::Down);
                    } else {
                        // It's part of identifier (e.g., "value")
                        slot_chars.push(ch);
                    }
                }
                'a'..='u' | 'w'..='z' | 'A'..='Z' | '0'..='9' => {
                    slot_chars.push(ch);
                }
                _ => {
                    // Unknown character, ignore
                }
            }
        }

        if !slot_chars.is_empty() {
            slot_name = Some(slot_chars);
        }

        LayoutCell {
            slot_name,
            h_align,
            v_align,
            merge,
        }
    }

    /// Get a span for the current line
    fn line_span(&self, line_idx: usize) -> Span {
        // Calculate approximate offset for this line
        let mut offset = 0u32;
        for (i, line) in self.lines.iter().enumerate() {
            if i == line_idx {
                break;
            }
            offset += line.len() as u32 + 1; // +1 for newline
        }
        let line_len = self.lines.get(line_idx).map(|l| l.len()).unwrap_or(0) as u32;
        Span::new(self.base_offset + offset, self.base_offset + offset + line_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_layout(content: &str) -> (Option<LayoutStmt>, Diagnostics) {
        let mut diagnostics = Diagnostics::new();
        let mut parser = LayoutParser::new(content, 0, &mut diagnostics);
        let result = parser.parse();
        (result, diagnostics)
    }

    #[test]
    fn test_basic_grid() {
        let (grid, diags) = parse_layout(
            r#"
| slot1 | slot2 |
| slot3 | slot4 |
"#,
        );
        assert!(!diags.has_errors());
        let grid = grid.unwrap();
        assert_eq!(grid.rows.len(), 2);
        assert_eq!(grid.rows[0].cells.len(), 2);
        assert_eq!(
            grid.rows[0].cells[0].slot_name,
            Some("slot1".to_string())
        );
    }

    #[test]
    fn test_column_sizes() {
        let (grid, diags) = parse_layout(
            r#"
  ~0.5    ~0.8
| slot1 | slot2 |
"#,
        );
        assert!(!diags.has_errors());
        let grid = grid.unwrap();
        assert_eq!(grid.column_sizes.len(), 2);
        match &grid.column_sizes[0] {
            LayoutSize::Weight(w) => assert!((w - 0.5).abs() < 0.001),
            _ => panic!("expected weight"),
        }
    }

    #[test]
    fn test_row_sizes() {
        let (grid, diags) = parse_layout(
            r#"
  24 | slot1 | slot2 |
 ~1  | slot3 | slot4 |
"#,
        );
        assert!(!diags.has_errors());
        let grid = grid.unwrap();
        assert_eq!(grid.rows.len(), 2);
        assert!(matches!(grid.rows[0].size, Some(LayoutSize::Fixed(24))));
        assert!(matches!(grid.rows[1].size, Some(LayoutSize::Weight(_))));
    }

    #[test]
    fn test_alignment_modifiers() {
        let (grid, diags) = parse_layout(
            r#"
| < slot1 | ! slot2 | > slot3 |
"#,
        );
        assert!(!diags.has_errors());
        let grid = grid.unwrap();
        assert_eq!(grid.rows[0].cells[0].h_align, HAlign::Left);
        assert_eq!(grid.rows[0].cells[1].h_align, HAlign::Center);
        assert_eq!(grid.rows[0].cells[2].h_align, HAlign::Right);
    }

    #[test]
    fn test_merge_indicators() {
        let (grid, diags) = parse_layout(
            r#"
| header | <--    |
| v--    | slot2  |
"#,
        );
        assert!(!diags.has_errors());
        let grid = grid.unwrap();
        assert_eq!(
            grid.rows[0].cells[1].merge,
            Some(MergeDirection::Left)
        );
        assert_eq!(
            grid.rows[1].cells[0].merge,
            Some(MergeDirection::Down)
        );
    }

    #[test]
    fn test_content_size() {
        let (grid, diags) = parse_layout(
            r#"
     #      ~1
  # | slot1 | slot2 |
"#,
        );
        assert!(!diags.has_errors());
        let grid = grid.unwrap();
        assert!(matches!(grid.column_sizes[0], LayoutSize::Content));
        assert!(matches!(grid.rows[0].size, Some(LayoutSize::Content)));
    }

    #[test]
    fn test_empty_cells() {
        let (grid, diags) = parse_layout(
            r#"
|        | slot |
| slot2  |      |
"#,
        );
        assert!(!diags.has_errors());
        let grid = grid.unwrap();
        assert!(grid.rows[0].cells[0].slot_name.is_none());
        assert!(grid.rows[1].cells[1].slot_name.is_none());
    }

    #[test]
    fn test_mismatched_columns() {
        let (grid, diags) = parse_layout(
            r#"
| a | b | c |
| d | e |
"#,
        );
        assert!(diags.has_errors());
        // Should still produce a grid
        assert!(grid.is_some());
    }
}
