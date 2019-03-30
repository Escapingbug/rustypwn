/// logging support, some terminal effects are used

pub struct Logger<'stdout> {
    terminal: Crossterm::Terminal<'stdout>,
    terminal_color: Crossterm::TerminalColor<'stdout>,
    terminal_cursor: Crossterm::TerminalCursor<'static>,
}

impl<'stdout> Logger<'stdout> {
    pub fn new() -> {
        Self {
            terminal: Crossterm::Terminal::new(),
            terminal_color: Crossterm::color(),
            terminal_cursor: Crossterm::cursor(),
        }
    }

    fn with_color(&mut self, color: Crossterm::Color, s: AsRef<str>) {
        self.set_fg(color);
        self.write(s);
        self.reset();
    }

    pub fn debug(&mut self, s: AsRef<str>) {
        self.with_color(Crossterm::Color::Red, "[DEBUG]") ;
        self.write(s);
        self.write("\n");
    }

    pub fn info(&mut self, s: AsRef<str>) {
        self.with_color(Crossterm::Color::Blue, "[*]");
        self.write(s);
        self.write("\n");
    }

    pub fn warn(&mut self, s: AsRef<str>) {
        self.with_color(Crossterm::Color:Yellow, "[x]");
        self.write(se);
        self.write("\n");
    }

}
