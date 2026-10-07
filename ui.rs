use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, Mode};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const SPACES_256: &str = "                                                                                                                                                                                                                                                                ";

pub fn draw(app: &App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(f.area());

    draw_editor(app, f, chunks[0]);
    draw_status_line(app, f, chunks[1]);
}

fn mode_line_bg(mode: Mode) -> Color {
    match mode {
        Mode::Insert => Color::Rgb(22, 38, 54),
        Mode::Normal => Color::Rgb(22, 28, 38),
        Mode::Visual => Color::Rgb(38, 20, 40),
        Mode::Command => Color::Rgb(42, 36, 18),
        Mode::Search => Color::Rgb(40, 36, 16),
    }
}

fn draw_editor(app: &App, f: &mut Frame, area: Rect) {
    let (cursor_line, cursor_col) = app.editor.cursor_line_col();
    let current_line_bg = mode_line_bg(app.mode);

    let is_visual = app.mode == Mode::Visual;
    let (sel_start, sel_end) = if is_visual {
        let a = app.vim_state.visual_anchor;
        let c = app.editor.cursor_char_idx;
        if a <= c { (a, c) } else { (c, a) }
    } else {
        (0, 0)
    };

    let visual_style = Style::default()
        .bg(Color::Rgb(70, 75, 120))
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    let start_line = app.editor.scroll_y;
    let visible_height = area.height as usize;
    let total_lines = app.editor.text.len_lines();

    let mut text_lines: Vec<Line> = Vec::with_capacity(visible_height);

    for row_idx in 0..visible_height {
        let line_idx = start_line + row_idx;
        if line_idx < total_lines {
            let line = app.editor.text.line(line_idx);
            let mut char_len = line.len_chars();
            if char_len > 0 && line.char(char_len - 1) == '\n' {
                char_len -= 1;
                if char_len > 0 && line.char(char_len - 1) == '\r' {
                    char_len -= 1;
                }
            }

            let line_slice = line.slice(..char_len);
            let is_current = line_idx == cursor_line;

            let line_style = if is_current {
                Style::default().bg(current_line_bg)
            } else {
                Style::default()
            };

            let mut spans: Vec<Span> = Vec::new();
            let mut visual_width: usize = 0;
            let line_start_char = app.editor.text.line_to_char(line_idx);

            if is_visual && line_start_char + char_len >= sel_start && line_start_char <= sel_end {
                let local_start = sel_start.saturating_sub(line_start_char).min(char_len);
                let local_end = (sel_end + 1).saturating_sub(line_start_char).min(char_len);

                if local_start > 0 {
                    let pre = line_slice.slice(..local_start);
                    for chunk in pre.chunks() {
                        visual_width += UnicodeWidthStr::width(chunk);
                        spans.push(Span::styled(chunk, line_style));
                    }
                }

                if local_start < local_end {
                    let mid = line_slice.slice(local_start..local_end);
                    for chunk in mid.chunks() {
                        visual_width += UnicodeWidthStr::width(chunk);
                        spans.push(Span::styled(chunk, visual_style));
                    }
                }

                if local_end < char_len {
                    let post = line_slice.slice(local_end..);
                    for chunk in post.chunks() {
                        visual_width += UnicodeWidthStr::width(chunk);
                        spans.push(Span::styled(chunk, line_style));
                    }
                }
            } else {
                for chunk in line_slice.chunks() {
                    visual_width += UnicodeWidthStr::width(chunk);
                    spans.push(Span::styled(chunk, line_style));
                }
            }

            // Fill remainder of current line
            if is_current {
                let width = area.width as usize;
                if visual_width < width {
                    let mut remaining = width - visual_width;
                    while remaining > 0 {
                        let fill_len = remaining.min(SPACES_256.len());
                        spans.push(Span::styled(&SPACES_256[..fill_len], line_style));
                        remaining -= fill_len;
                    }
                }
            }

            text_lines.push(Line::from(spans));
        } else {
            text_lines.push(Line::default());
        }
    }

    let p = Paragraph::new(text_lines);
    f.render_widget(p, area);

    // Position cursor
    if cursor_line >= app.editor.scroll_y && cursor_line - app.editor.scroll_y < area.height as usize {
        let line = app.editor.text.line(cursor_line);
        let mut visual_x: u16 = 0;
        for c in line.chars().take(cursor_col) {
            visual_x += UnicodeWidthChar::width(c).unwrap_or(0) as u16;
        }

        let screen_y = area.y + (cursor_line - app.editor.scroll_y) as u16;
        let screen_x = (area.x + visual_x).min(area.x + area.width.saturating_sub(1));

        if app.mode != Mode::Command && app.mode != Mode::Search {
            f.set_cursor_position(ratatui::layout::Position { x: screen_x, y: screen_y });
        }
    }
}

fn draw_status_line(app: &App, f: &mut Frame, area: Rect) {
    let mode_str = match app.mode {
        Mode::Normal => " NORMAL ",
        Mode::Insert => " INSERT ",
        Mode::Command => " COMMAND ",
        Mode::Search => " SEARCH ",
        Mode::Visual => " VISUAL ",
    };

    let mode_style = match app.mode {
        Mode::Normal => Style::default()
            .bg(Color::Rgb(100, 149, 237))
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
        Mode::Insert => Style::default()
            .bg(Color::Rgb(135, 206, 250))
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
        Mode::Command => Style::default()
            .bg(Color::Rgb(255, 191, 0))
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
        Mode::Search => Style::default()
            .bg(Color::Rgb(255, 215, 0))
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
        Mode::Visual => Style::default()
            .bg(Color::Rgb(218, 112, 214))
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    };

    let file_str = app.editor.file_path.as_deref().unwrap_or("[No Name]");
    let (line, col) = app.editor.cursor_line_col();
    let pos_str = format!("Ln {}, Col {}", line + 1, col + 1);

    let status_text = if app.mode == Mode::Command {
        format!(":{}", app.command_buffer)
    } else if app.mode == Mode::Search {
        format!("/{}", app.command_buffer)
    } else if let Some(err) = &app.error_msg {
        format!("Error: {}", err)
    } else {
        let mut extra = String::new();
        if let Some(c) = app.vim_state.count {
            extra.push_str(&c.to_string());
        }
        if let Some(op) = app.vim_state.pending_operator {
            match op {
                crate::vim::Operator::Delete => extra.push('d'),
                crate::vim::Operator::Yank => extra.push('y'),
                crate::vim::Operator::Change => extra.push('c'),
            }
        }
        if extra.is_empty() {
            format!(" {} ", file_str)
        } else {
            format!(" {} [{}] ", file_str, extra)
        }
    };

    let process_running = crate::dll_lib::is_process_running("RobloxPlayerBeta.exe");
    let is_attached = process_running && crate::dll_lib::is_attached();

    let (circle_char, circle_color) = if is_attached {
        ("●", Color::LightGreen)
    } else if process_running {
        ("●", Color::Yellow)
    } else {
        ("●", Color::Red)
    };

    let mut spans = vec![Span::styled(mode_str, mode_style)];
    spans.push(Span::raw(format!(" {} | {} ", status_text, pos_str)));
    spans.push(Span::styled(circle_char, Style::default().fg(circle_color)));

    if let Some(log) = &app.attach_log {
        spans.push(Span::styled(
            format!(" [{}]", log),
            Style::default().fg(Color::LightRed),
        ));
    }

    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);

    if app.mode == Mode::Command || app.mode == Mode::Search {
        let visual_x = mode_str.len() as u16 + 2 + app.command_buffer.chars().count() as u16;
        f.set_cursor_position(ratatui::layout::Position { x: area.x + visual_x, y: area.y });
    }
}
