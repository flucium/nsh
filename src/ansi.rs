pub enum Cursor {
    Move(usize),
    Backspace,
    Left,
    Right,
}

impl Cursor {
    pub fn get_esc_code(&self) -> String {
        return match &self {
            Cursor::Move(position) => format!("\x1b[{position}G"),
            Cursor::Backspace => format!("\x08{}", " "),
            Cursor::Left => format!("\x1b[1D"),
            Cursor::Right => format!("\x1b[1C"),
        };
    }
}
