use crossterm::event::KeyEvent;
use ratatui::Frame;
use crate::editor::Editor;
use crate::vim::VimState;
use crate::ui;

pub enum Message {
    Key(KeyEvent),
    Resize(u16, u16),
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Search,
}

pub struct App {
    pub mode: Mode,
    pub editor: Editor,
    pub vim_state: VimState,
    pub command_buffer: String,
    pub should_quit: bool,
    pub error_msg: Option<String>,
    pub attach_log: Option<String>,
    pub term_width: u16,
    pub term_height: u16,
}

impl App {
    pub fn new() -> App {
        App {
            mode: Mode::Normal,
            editor: Editor::new(),
            vim_state: VimState::new(),
            command_buffer: String::new(),
            should_quit: false,
            error_msg: None,
            attach_log: None,
            term_width: 80,
            term_height: 10,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Key(key) => {
                crate::vim::handle_key(self, key);
                self.update_scroll();
            }
            Message::Resize(w, h) => {
                self.term_width = w;
                self.term_height = h;
                self.update_scroll();
            }
            Message::Quit => {
                self.should_quit = true;
            }
        }
    }

    fn update_scroll(&mut self) {
        let (cursor_line, _cursor_col) = self.editor.cursor_line_col();
        // Assume 1 line for status bar
        let editor_height = self.term_height.saturating_sub(1) as usize;
        
        if cursor_line < self.editor.scroll_y {
            self.editor.scroll_y = cursor_line;
        } else if editor_height > 0 && cursor_line >= self.editor.scroll_y + editor_height {
            self.editor.scroll_y = cursor_line - editor_height + 1;
        }
    }

    pub fn view(&self, f: &mut Frame) {
        ui::draw(self, f);
    }
}
