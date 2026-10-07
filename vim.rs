use crate::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct VimState {
    pub pending_operator: Option<Operator>,
    pub pending_g: bool,
    pub replace_char_mode: bool,
    pub count: Option<usize>,
    pub visual_anchor: usize,
    pub register: String,
    pub last_search: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
}

impl VimState {
    pub fn new() -> Self {
        Self {
            pending_operator: None,
            pending_g: false,
            replace_char_mode: false,
            count: None,
            visual_anchor: 0,
            register: String::new(),
            last_search: String::new(),
        }
    }

    pub fn take_count(&mut self) -> usize {
        self.count.take().unwrap_or(1)
    }

    pub fn reset_pending(&mut self) {
        self.pending_operator = None;
        self.pending_g = false;
        self.replace_char_mode = false;
        self.count = None;
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    app.error_msg = None;

    if app.vim_state.replace_char_mode {
        if let KeyCode::Char(c) = key.code {
            app.editor.save_history();
            if app.editor.cursor_char_idx < app.editor.text.len_chars() {
                let idx = app.editor.cursor_char_idx;
                app.editor.text.remove(idx..idx + 1);
                app.editor.text.insert_char(idx, c);
            }
        }
        app.vim_state.replace_char_mode = false;
        return;
    }

    match app.mode {
        Mode::Normal => handle_normal_key(app, key),
        Mode::Insert => handle_insert_key(app, key),
        Mode::Command => handle_command_key(app, key),
        Mode::Search => handle_search_key(app, key),
        Mode::Visual => handle_visual_key(app, key),
    }
}

fn apply_motion<F>(app: &mut App, motion_fn: F)
where
    F: FnOnce(&mut App),
{
    let op = app.vim_state.pending_operator.take();
    let start_pos = app.editor.cursor_char_idx;
    motion_fn(app);
    let end_pos = app.editor.cursor_char_idx;

    match op {
        Some(Operator::Delete) => {
            let yanked = app.editor.delete_range(start_pos, end_pos);
            app.vim_state.register = yanked;
        }
        Some(Operator::Yank) => {
            app.vim_state.register = app.editor.yank_range(start_pos, end_pos);
            app.editor.cursor_char_idx = start_pos;
        }
        Some(Operator::Change) => {
            let yanked = app.editor.delete_range(start_pos, end_pos);
            app.vim_state.register = yanked;
            app.mode = Mode::Insert;
        }
        None => {}
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent) {
    // Digit prefix accumulation: 1-9 starts a count; 0 appends if count already active
    if let KeyCode::Char(c) = key.code {
        if (c.is_ascii_digit() && c != '0') || (c == '0' && app.vim_state.count.is_some()) {
            let digit = c.to_digit(10).unwrap() as usize;
            let current = app.vim_state.count.unwrap_or(0);
            app.vim_state.count = Some(current * 10 + digit);
            return;
        }
    }

    // Handle 'g' prefix (e.g. gg)
    if app.vim_state.pending_g {
        app.vim_state.pending_g = false;
        if key.code == KeyCode::Char('g') {
            let count = app.vim_state.count.take();
            apply_motion(app, |a| a.editor.move_to_first_line(count));
            return;
        }
    }

    match key.code {
        KeyCode::Esc => {
            app.vim_state.reset_pending();
        }
        // Movement motions
        KeyCode::Char('h') | KeyCode::Left => {
            let count = app.vim_state.take_count();
            apply_motion(app, |a| {
                for _ in 0..count {
                    a.editor.move_left(false);
                }
            });
        }
        KeyCode::Char('l') | KeyCode::Right => {
            let count = app.vim_state.take_count();
            apply_motion(app, |a| {
                for _ in 0..count {
                    a.editor.move_right(false);
                }
            });
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let count = app.vim_state.take_count();
            apply_motion(app, |a| {
                for _ in 0..count {
                    a.editor.move_down(false);
                }
            });
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let count = app.vim_state.take_count();
            apply_motion(app, |a| {
                for _ in 0..count {
                    a.editor.move_up(false);
                }
            });
        }
        KeyCode::Char('w') => {
            let count = app.vim_state.take_count();
            apply_motion(app, |a| a.editor.move_word_forward(count));
        }
        KeyCode::Char('b') => {
            let count = app.vim_state.take_count();
            apply_motion(app, |a| a.editor.move_word_backward(count));
        }
        KeyCode::Char('e') => {
            let count = app.vim_state.take_count();
            apply_motion(app, |a| a.editor.move_word_end(count));
        }
        KeyCode::Char('0') => {
            apply_motion(app, |a| a.editor.move_to_start_of_line());
        }
        KeyCode::Char('^') => {
            apply_motion(app, |a| a.editor.move_to_first_non_whitespace());
        }
        KeyCode::Char('$') => {
            apply_motion(app, |a| a.editor.move_to_end_of_line(false));
        }
        KeyCode::Char('G') => {
            let count = app.vim_state.count.take();
            apply_motion(app, |a| a.editor.move_to_last_line(count));
        }
        KeyCode::Char('g') => {
            app.vim_state.pending_g = true;
        }
        KeyCode::Char('{') => {
            apply_motion(app, |a| a.editor.jump_paragraph_backward());
        }
        KeyCode::Char('}') => {
            apply_motion(app, |a| a.editor.jump_paragraph_forward());
        }

        // Change operator & mode ('c')
        KeyCode::Char('c') => {
            if let Some(Operator::Change) = app.vim_state.pending_operator {
                // 'cc' changes whole line
                let line_idx = app.editor.cursor_line_idx();
                let start = app.editor.text.line_to_char(line_idx);
                let end = start + app.editor.get_line_len(line_idx);
                app.vim_state.register = app.editor.delete_range(start, end);
                app.vim_state.reset_pending();
                app.mode = Mode::Insert;
            } else {
                app.vim_state.pending_operator = Some(Operator::Change);
            }
        }
        KeyCode::Char('C') => {
            // 'C' = change to end of line ('c$')
            let start = app.editor.cursor_char_idx;
            app.editor.move_to_end_of_line(false);
            let end = app.editor.cursor_char_idx + 1;
            app.vim_state.register = app.editor.delete_range(start, end);
            app.vim_state.reset_pending();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('s') => {
            // Substitute char: delete 1 char and enter Insert mode
            app.editor.delete_char_forward();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('S') => {
            // Substitute line: delete line and enter Insert mode
            let line_idx = app.editor.cursor_line_idx();
            let start = app.editor.text.line_to_char(line_idx);
            let end = start + app.editor.get_line_len(line_idx);
            app.vim_state.register = app.editor.delete_range(start, end);
            app.mode = Mode::Insert;
        }

        // Delete operator ('d')
        KeyCode::Char('d') => {
            if let Some(Operator::Delete) = app.vim_state.pending_operator {
                // 'dd' deletes lines
                let count = app.vim_state.take_count();
                for _ in 0..count {
                    let line_idx = app.editor.cursor_line_idx();
                    if line_idx < app.editor.text.len_lines() {
                        let start = app.editor.text.line_to_char(line_idx);
                        let end = start + app.editor.text.line(line_idx).len_chars();
                        app.vim_state.register = app.editor.yank_line();
                        app.editor.save_history();
                        app.editor.text.remove(start..end);
                    }
                }
                let len = app.editor.text.len_chars();
                if app.editor.cursor_char_idx >= len && len > 0 {
                    app.editor.cursor_char_idx = len - 1;
                }
                app.vim_state.reset_pending();
            } else {
                app.vim_state.pending_operator = Some(Operator::Delete);
            }
        }
        KeyCode::Char('D') => {
            // Delete to end of line ('d$')
            let start = app.editor.cursor_char_idx;
            app.editor.move_to_end_of_line(false);
            let end = app.editor.cursor_char_idx + 1;
            app.vim_state.register = app.editor.delete_range(start, end);
            app.vim_state.reset_pending();
        }
        KeyCode::Char('x') => {
            let count = app.vim_state.take_count();
            for _ in 0..count {
                app.editor.delete_char_forward();
            }
        }
        KeyCode::Char('X') => {
            let count = app.vim_state.take_count();
            for _ in 0..count {
                app.editor.delete_char_backward();
            }
        }

        // Yank operator ('y')
        KeyCode::Char('y') => {
            if let Some(Operator::Yank) = app.vim_state.pending_operator {
                app.vim_state.register = app.editor.yank_line();
                app.vim_state.reset_pending();
            } else {
                app.vim_state.pending_operator = Some(Operator::Yank);
            }
        }
        KeyCode::Char('p') => {
            if !app.vim_state.register.is_empty() {
                let reg = app.vim_state.register.clone();
                app.editor.insert_str(&reg);
            }
        }

        // Insert triggers
        KeyCode::Char('i') => {
            app.editor.end_edit_group();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('a') => {
            app.editor.move_right(true);
            app.editor.end_edit_group();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('A') => {
            app.editor.move_to_end_of_line(true);
            app.editor.end_edit_group();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('I') => {
            app.editor.move_to_first_non_whitespace();
            app.editor.end_edit_group();
            app.mode = Mode::Insert;
        }
        KeyCode::Char('o') => {
            app.editor.move_to_end_of_line(true);
            app.editor.insert_char('\n');
            app.mode = Mode::Insert;
        }
        KeyCode::Char('O') => {
            app.editor.move_to_start_of_line();
            app.editor.insert_char('\n');
            app.editor.move_left(true);
            app.mode = Mode::Insert;
        }

        // Single replace
        KeyCode::Char('r') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.vim_state.replace_char_mode = true;
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.editor.redo();
        }
        KeyCode::Char('u') => {
            app.editor.undo();
        }
        KeyCode::Char('~') => {
            app.editor.toggle_case();
        }
        KeyCode::Char('J') => {
            app.editor.join_lines();
        }

        // Visual mode
        KeyCode::Char('v') => {
            app.mode = Mode::Visual;
            app.vim_state.visual_anchor = app.editor.cursor_char_idx;
        }

        // Command mode: ONLY triggered by ':'
        KeyCode::Char(':') => {
            app.mode = Mode::Command;
            app.command_buffer.clear();
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Search;
            app.command_buffer.clear();
        }
        KeyCode::Char('n') => {
            find_next_match(app);
        }
        _ => {}
    }
}

fn handle_insert_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char(';') => {
            app.editor.end_edit_group();
            app.mode = Mode::Normal;
            app.editor.move_left(false);
        }
        KeyCode::Char(c) => app.editor.insert_char(c),
        KeyCode::Enter => app.editor.insert_char('\n'),
        KeyCode::Backspace => app.editor.delete_char_backward(),
        KeyCode::Delete => app.editor.delete_char_forward(),
        KeyCode::Up => app.editor.move_up(true),
        KeyCode::Down => app.editor.move_down(true),
        KeyCode::Left => app.editor.move_left(true),
        KeyCode::Right => app.editor.move_right(true),
        _ => {}
    }
}

fn handle_command_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = Mode::Normal,
        KeyCode::Char(c) => app.command_buffer.push(c),
        KeyCode::Backspace => {
            if app.command_buffer.pop().is_none() {
                app.mode = Mode::Normal;
            }
        }
        KeyCode::Enter => {
            execute_command(app);
            app.mode = Mode::Normal;
        }
        _ => {}
    }
}

fn execute_command(app: &mut App) {
    let cmd = app.command_buffer.trim();
    if cmd == "q" || cmd == "qa" {
        app.should_quit = true;
    } else if cmd == "attach" || cmd == "attach;" {
        match crate::dll_lib::attach() {
            Ok(()) => {
                app.attach_log = None;
                app.error_msg = Some("Attached successfully".to_string());
            }
            Err(e) => {
                app.attach_log = Some(e.clone());
                app.error_msg = Some(format!("Attach fail: {}", e));
            }
        }
    } else if cmd.eq_ignore_ascii_case("isAttached") {
        let attached = crate::dll_lib::is_attached();
        app.error_msg = Some(format!("isAttached: {}", attached));
    } else if cmd == "exe" || cmd == "exec" || cmd == "execute" {
        let script = app.editor.text.to_string();
        match crate::dll_lib::execute(&script) {
            Ok(()) => {
                app.error_msg = Some("Script executed".to_string());
            }
            Err(e) => {
                app.error_msg = Some(format!("Execute fail: {}", e));
            }
        }
    } else if cmd == "w" {
        if let Err(e) = app.editor.save_file() {
            app.error_msg = Some(format!("Save failed: {}", e));
        }
    } else if cmd == "wq" {
        if let Err(e) = app.editor.save_file() {
            app.error_msg = Some(format!("Save failed: {}", e));
        } else {
            app.should_quit = true;
        }
    } else if let Some(path) = cmd.strip_prefix("w ") {
        app.editor.file_path = Some(path.trim().to_string());
        if let Err(e) = app.editor.save_file() {
            app.error_msg = Some(format!("Save failed: {}", e));
        }
    } else if let Some(path) = cmd.strip_prefix("e ") {
        if let Err(e) = app.editor.load_file(path.trim()) {
            app.error_msg = Some(format!("Load failed: {}", e));
        }
    } else {
        app.error_msg = Some(format!("Unknown command: {}", cmd));
    }
}

fn handle_search_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = Mode::Normal,
        KeyCode::Char(c) => app.command_buffer.push(c),
        KeyCode::Backspace => {
            if app.command_buffer.pop().is_none() {
                app.mode = Mode::Normal;
            }
        }
        KeyCode::Enter => {
            app.vim_state.last_search = app.command_buffer.clone();
            find_next_match(app);
            app.mode = Mode::Normal;
        }
        _ => {}
    }
}

fn find_next_match(app: &mut App) {
    if app.vim_state.last_search.is_empty() {
        return;
    }
    let query = app.vim_state.last_search.clone();
    let text_str = app.editor.text.to_string();
    let start_idx = app.editor.cursor_char_idx + 1;

    let found = if start_idx < text_str.chars().count() {
        let after_slice: String = text_str.chars().skip(start_idx).collect();
        after_slice.find(&query).map(|byte_pos| {
            let char_offset = after_slice[..byte_pos].chars().count();
            start_idx + char_offset
        })
    } else {
        None
    };

    let target = found.or_else(|| {
        text_str
            .find(&query)
            .map(|byte_pos| text_str[..byte_pos].chars().count())
    });

    if let Some(pos) = target {
        app.editor.cursor_char_idx = pos;
    } else {
        app.error_msg = Some(format!("Pattern not found: {}", query));
    }
}

fn handle_visual_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char(';') => {
            app.mode = Mode::Normal;
        }
        KeyCode::Char('h') | KeyCode::Left => app.editor.move_left(false),
        KeyCode::Char('j') | KeyCode::Down => app.editor.move_down(false),
        KeyCode::Char('k') | KeyCode::Up => app.editor.move_up(false),
        KeyCode::Char('l') | KeyCode::Right => app.editor.move_right(false),
        KeyCode::Char('w') => app.editor.move_word_forward(1),
        KeyCode::Char('b') => app.editor.move_word_backward(1),
        KeyCode::Char('e') => app.editor.move_word_end(1),
        KeyCode::Char('$') => app.editor.move_to_end_of_line(false),
        KeyCode::Char('0') => app.editor.move_to_start_of_line(),
        KeyCode::Char('y') => {
            let anchor = app.vim_state.visual_anchor;
            let cursor = app.editor.cursor_char_idx;
            app.vim_state.register = app.editor.yank_range(anchor, cursor + 1);
            app.mode = Mode::Normal;
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            let anchor = app.vim_state.visual_anchor;
            let cursor = app.editor.cursor_char_idx;
            app.vim_state.register = app.editor.delete_range(anchor, cursor + 1);
            app.mode = Mode::Normal;
        }
        KeyCode::Char('c') => {
            // 'c' in visual mode deletes range and enters Insert mode!
            let anchor = app.vim_state.visual_anchor;
            let cursor = app.editor.cursor_char_idx;
            app.vim_state.register = app.editor.delete_range(anchor, cursor + 1);
            app.mode = Mode::Insert;
        }
        _ => {}
    }
}
