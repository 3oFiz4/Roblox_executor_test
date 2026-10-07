use ropey::Rope;
use std::collections::VecDeque;
use std::fs;

const MAX_HISTORY: usize = 100;

pub struct Editor {
    pub text: Rope,
    pub cursor_char_idx: usize,
    pub scroll_y: usize,
    pub scroll_x: usize,
    undo_stack: VecDeque<(Rope, usize)>,
    redo_stack: VecDeque<(Rope, usize)>,
    pub in_edit_group: bool,
    pub file_path: Option<String>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            text: Rope::from(""),
            cursor_char_idx: 0,
            scroll_y: 0,
            scroll_x: 0,
            undo_stack: VecDeque::with_capacity(MAX_HISTORY),
            redo_stack: VecDeque::with_capacity(MAX_HISTORY),
            in_edit_group: false,
            file_path: None,
        }
    }

    pub fn load_file(&mut self, path: &str) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.text = Rope::from_str(&content);
        self.file_path = Some(path.to_string());
        self.cursor_char_idx = 0;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.in_edit_group = false;
        Ok(())
    }

    pub fn save_file(&self) -> std::io::Result<()> {
        if let Some(path) = &self.file_path {
            fs::write(path, self.text.to_string())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file name specified",
            ))
        }
    }

    pub fn end_edit_group(&mut self) {
        self.in_edit_group = false;
    }

    pub fn save_history(&mut self) {
        if self.undo_stack.len() >= MAX_HISTORY {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back((self.text.clone(), self.cursor_char_idx));
        self.redo_stack.clear();
    }

    pub fn insert_char(&mut self, ch: char) {
        if !self.in_edit_group {
            self.save_history();
            self.in_edit_group = true;
        }
        self.text.insert_char(self.cursor_char_idx, ch);
        self.cursor_char_idx += 1;
        if ch == '\n' {
            self.in_edit_group = false;
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        self.end_edit_group();
        self.save_history();
        self.text.insert(self.cursor_char_idx, s);
        self.cursor_char_idx += s.chars().count();
    }

    pub fn delete_char_backward(&mut self) {
        if self.cursor_char_idx > 0 {
            if !self.in_edit_group {
                self.save_history();
                self.in_edit_group = true;
            }
            self.cursor_char_idx -= 1;
            self.text.remove(self.cursor_char_idx..self.cursor_char_idx + 1);
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_char_idx < self.text.len_chars() {
            self.end_edit_group();
            self.save_history();
            self.text.remove(self.cursor_char_idx..self.cursor_char_idx + 1);
            let len = self.text.len_chars();
            if self.cursor_char_idx >= len && len > 0 {
                self.cursor_char_idx = len - 1;
            }
        }
    }

    pub fn delete_range(&mut self, start: usize, end: usize) -> String {
        self.end_edit_group();
        let (s, e) = if start <= end { (start, end) } else { (end, start) };
        let total = self.text.len_chars();
        let s = s.min(total);
        let e = e.min(total);
        if s < e {
            self.save_history();
            let yanked = self.text.slice(s..e).to_string();
            self.text.remove(s..e);
            let len = self.text.len_chars();
            self.cursor_char_idx = if s >= len && len > 0 { len - 1 } else { s };
            yanked
        } else {
            String::new()
        }
    }

    pub fn yank_range(&self, start: usize, end: usize) -> String {
        let (s, e) = if start <= end { (start, end) } else { (end, start) };
        let total = self.text.len_chars();
        let s = s.min(total);
        let e = e.min(total);
        if s < e {
            self.text.slice(s..e).to_string()
        } else {
            String::new()
        }
    }

    pub fn undo(&mut self) {
        self.end_edit_group();
        if let Some((prev_text, prev_cursor)) = self.undo_stack.pop_back() {
            if self.redo_stack.len() >= MAX_HISTORY {
                self.redo_stack.pop_front();
            }
            self.redo_stack.push_back((self.text.clone(), self.cursor_char_idx));
            self.text = prev_text;
            self.cursor_char_idx = prev_cursor;
        }
    }

    pub fn redo(&mut self) {
        self.end_edit_group();
        if let Some((next_text, next_cursor)) = self.redo_stack.pop_back() {
            if self.undo_stack.len() >= MAX_HISTORY {
                self.undo_stack.pop_front();
            }
            self.undo_stack.push_back((self.text.clone(), self.cursor_char_idx));
            self.text = next_text;
            self.cursor_char_idx = next_cursor;
        }
    }

    pub fn cursor_line_idx(&self) -> usize {
        let len = self.text.len_chars();
        let idx = self.cursor_char_idx.min(len);
        self.text.char_to_line(idx)
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        let len = self.text.len_chars();
        let idx = self.cursor_char_idx.min(len);
        let line_idx = self.text.char_to_line(idx);
        let line_char_idx = self.text.line_to_char(line_idx);
        (line_idx, idx.saturating_sub(line_char_idx))
    }

    pub fn get_line_len(&self, line_idx: usize) -> usize {
        if line_idx >= self.text.len_lines() {
            return 0;
        }
        let line = self.text.line(line_idx);
        let mut line_len = line.len_chars();
        if line.chars().last() == Some('\n') {
            line_len -= 1;
        }
        if line_len > 0 && line.chars().nth(line_len.saturating_sub(1)) == Some('\r') {
            line_len -= 1;
        }
        line_len
    }

    pub fn move_left(&mut self, is_insert: bool) {
        self.end_edit_group();
        let (line_idx, col) = self.cursor_line_col();
        if col > 0 {
            self.cursor_char_idx -= 1;
        } else if is_insert && line_idx > 0 {
            self.move_up(is_insert);
            let new_line_idx = self.cursor_line_idx();
            let target_line_start = self.text.line_to_char(new_line_idx);
            let len = self.get_line_len(new_line_idx);
            self.cursor_char_idx = target_line_start + len;
        }
    }

    pub fn move_right(&mut self, is_insert: bool) {
        self.end_edit_group();
        let (line_idx, col) = self.cursor_line_col();
        let len = self.get_line_len(line_idx);
        let max_col = if is_insert { len } else { len.saturating_sub(1) };
        
        if col < max_col {
            self.cursor_char_idx += 1;
        } else if is_insert && line_idx + 1 < self.text.len_lines() {
            self.move_down(is_insert);
            let new_line_idx = self.cursor_line_idx();
            self.cursor_char_idx = self.text.line_to_char(new_line_idx);
        }
    }

    pub fn move_up(&mut self, is_insert: bool) {
        self.end_edit_group();
        let (line_idx, col) = self.cursor_line_col();
        if line_idx > 0 {
            let target_line = line_idx - 1;
            let target_line_start = self.text.line_to_char(target_line);
            let target_line_len = self.get_line_len(target_line);
            let max_col = if is_insert { target_line_len } else { target_line_len.saturating_sub(1) };
            let target_col = col.min(max_col);
            self.cursor_char_idx = target_line_start + target_col;
        }
    }

    pub fn move_down(&mut self, is_insert: bool) {
        self.end_edit_group();
        let (line_idx, col) = self.cursor_line_col();
        if line_idx + 1 < self.text.len_lines() {
            let target_line = line_idx + 1;
            let target_line_start = self.text.line_to_char(target_line);
            let target_line_len = self.get_line_len(target_line);
            let max_col = if is_insert { target_line_len } else { target_line_len.saturating_sub(1) };
            let target_col = col.min(max_col);
            self.cursor_char_idx = target_line_start + target_col;
        }
    }
    
    pub fn move_to_start_of_line(&mut self) {
        self.end_edit_group();
        let line_idx = self.cursor_line_idx();
        self.cursor_char_idx = self.text.line_to_char(line_idx);
    }
    
    pub fn move_to_end_of_line(&mut self, is_insert: bool) {
        self.end_edit_group();
        let line_idx = self.cursor_line_idx();
        let start = self.text.line_to_char(line_idx);
        let len = self.get_line_len(line_idx);
        let max_col = if is_insert { len } else { len.saturating_sub(1) };
        self.cursor_char_idx = start + max_col;
    }

    pub fn move_to_first_non_whitespace(&mut self) {
        self.end_edit_group();
        let line_idx = self.cursor_line_idx();
        let start = self.text.line_to_char(line_idx);
        let len = self.get_line_len(line_idx);
        let mut offset = 0;
        while offset < len && self.text.char(start + offset).is_whitespace() {
            offset += 1;
        }
        self.cursor_char_idx = start + offset;
    }

    pub fn move_word_forward(&mut self, count: usize) {
        self.end_edit_group();
        let total = self.text.len_chars();
        for _ in 0..count {
            if self.cursor_char_idx >= total {
                break;
            }
            let mut idx = self.cursor_char_idx;
            let first_char = self.text.char(idx);
            let is_first_alnum = first_char.is_alphanumeric() || first_char == '_';
            let is_first_ws = first_char.is_whitespace();

            if is_first_ws {
                while idx < total && self.text.char(idx).is_whitespace() {
                    idx += 1;
                }
            } else if is_first_alnum {
                while idx < total && (self.text.char(idx).is_alphanumeric() || self.text.char(idx) == '_') {
                    idx += 1;
                }
                while idx < total && self.text.char(idx).is_whitespace() {
                    idx += 1;
                }
            } else {
                while idx < total && !self.text.char(idx).is_alphanumeric() && self.text.char(idx) != '_' && !self.text.char(idx).is_whitespace() {
                    idx += 1;
                }
                while idx < total && self.text.char(idx).is_whitespace() {
                    idx += 1;
                }
            }
            self.cursor_char_idx = idx.min(total.saturating_sub(1));
        }
    }

    pub fn move_word_backward(&mut self, count: usize) {
        self.end_edit_group();
        for _ in 0..count {
            if self.cursor_char_idx == 0 {
                break;
            }
            let mut idx = self.cursor_char_idx;
            while idx > 0 && self.text.char(idx - 1).is_whitespace() {
                idx -= 1;
            }
            if idx == 0 {
                self.cursor_char_idx = 0;
                break;
            }
            let prev_char = self.text.char(idx - 1);
            if prev_char.is_alphanumeric() || prev_char == '_' {
                while idx > 0 && (self.text.char(idx - 1).is_alphanumeric() || self.text.char(idx - 1) == '_') {
                    idx -= 1;
                }
            } else {
                while idx > 0 && !self.text.char(idx - 1).is_alphanumeric() && self.text.char(idx - 1) != '_' && !self.text.char(idx - 1).is_whitespace() {
                    idx -= 1;
                }
            }
            self.cursor_char_idx = idx;
        }
    }

    pub fn move_word_end(&mut self, count: usize) {
        self.end_edit_group();
        let total = self.text.len_chars();
        for _ in 0..count {
            if self.cursor_char_idx + 1 >= total {
                break;
            }
            let mut idx = self.cursor_char_idx + 1;
            while idx < total && self.text.char(idx).is_whitespace() {
                idx += 1;
            }
            if idx >= total {
                self.cursor_char_idx = total.saturating_sub(1);
                break;
            }
            let ch = self.text.char(idx);
            if ch.is_alphanumeric() || ch == '_' {
                while idx + 1 < total && (self.text.char(idx + 1).is_alphanumeric() || self.text.char(idx + 1) == '_') {
                    idx += 1;
                }
            } else {
                while idx + 1 < total && !self.text.char(idx + 1).is_alphanumeric() && self.text.char(idx + 1) != '_' && !self.text.char(idx + 1).is_whitespace() {
                    idx += 1;
                }
            }
            self.cursor_char_idx = idx;
        }
    }

    pub fn move_to_first_line(&mut self, line_opt: Option<usize>) {
        self.end_edit_group();
        let target_line = line_opt
            .map(|l| l.saturating_sub(1))
            .unwrap_or(0)
            .min(self.text.len_lines().saturating_sub(1));
        self.cursor_char_idx = self.text.line_to_char(target_line);
    }

    pub fn move_to_last_line(&mut self, line_opt: Option<usize>) {
        self.end_edit_group();
        let target_line = match line_opt {
            Some(l) => l.saturating_sub(1).min(self.text.len_lines().saturating_sub(1)),
            None => self.text.len_lines().saturating_sub(1),
        };
        self.cursor_char_idx = self.text.line_to_char(target_line);
    }

    pub fn jump_paragraph_forward(&mut self) {
        self.end_edit_group();
        let current_line = self.cursor_line_idx();
        let total = self.text.len_lines();
        let mut target = total.saturating_sub(1);
        for l in (current_line + 1)..total {
            if self.get_line_len(l) == 0 {
                target = l;
                break;
            }
        }
        self.cursor_char_idx = self.text.line_to_char(target);
    }

    pub fn jump_paragraph_backward(&mut self) {
        self.end_edit_group();
        let current_line = self.cursor_line_idx();
        let mut target = 0;
        for l in (0..current_line).rev() {
            if self.get_line_len(l) == 0 {
                target = l;
                break;
            }
        }
        self.cursor_char_idx = self.text.line_to_char(target);
    }

    pub fn toggle_case(&mut self) {
        if self.cursor_char_idx < self.text.len_chars() {
            self.save_history();
            let ch = self.text.char(self.cursor_char_idx);
            let toggled = if ch.is_uppercase() {
                ch.to_lowercase().next().unwrap_or(ch)
            } else {
                ch.to_uppercase().next().unwrap_or(ch)
            };
            self.text.remove(self.cursor_char_idx..self.cursor_char_idx + 1);
            self.text.insert_char(self.cursor_char_idx, toggled);
            self.move_right(false);
        }
    }

    pub fn join_lines(&mut self) {
        let line_idx = self.cursor_line_idx();
        if line_idx + 1 < self.text.len_lines() {
            self.save_history();
            let line = self.text.line(line_idx);
            let start = self.text.line_to_char(line_idx);
            let len = line.len_chars();
            let mut remove_start = start + len;
            if len > 0 && line.char(len - 1) == '\n' {
                remove_start -= 1;
                if len > 1 && line.char(len - 2) == '\r' {
                    remove_start -= 1;
                }
                self.text.remove(remove_start..start + len);
                self.text.insert_char(remove_start, ' ');
                self.cursor_char_idx = remove_start;
            }
        }
    }
    
    pub fn yank_line(&self) -> String {
        let line_idx = self.cursor_line_idx();
        if line_idx < self.text.len_lines() {
            self.text.line(line_idx).to_string()
        } else {
            String::new()
        }
    }
}
