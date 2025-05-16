use std::{cmp::{max, min}, io, ops::Index};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Layout, Rect}, symbols, widgets::{Block, Padding, Tabs, Widget}, DefaultTerminal, Frame};
use tab::{DeviceTab, OverviewTab, Tab};

mod tab;

#[derive(Default)]
pub struct App {
    exit: bool,
    tabs: Vec<Box<dyn Tab>>,
    tab: usize
}

impl App {

    pub fn new() -> Self {
        App {
            tabs: vec![Box::new(OverviewTab {}), Box::new(DeviceTab { name: "Device One".to_string() })],
            ..Default::default()
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(e) if e.kind == KeyEventKind::Press && e.code == KeyCode::Char('c') && e.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit = true;
            },
            Event::Key(e) if e.kind == KeyEventKind::Press && e.code == KeyCode::Left => {
                if self.tab > 0 {
                    self.tab -= 1;
                }
            },
            Event::Key(e) if e.kind == KeyEventKind::Press && e.code == KeyCode::Right => {
                if self.tab < self.tabs.len() - 1 {
                    self.tab += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::Constraint::{Length, Min};

        let layout = Layout::vertical([Min(0), Length(3)]);
        let [tab_area, title_area] = layout.areas(area);

        let titles = self.tabs.iter().map(Box::as_ref).map(Tab::title);

        let block = Block::bordered();

        Tabs::new(titles)
            .block(block)
            .select(self.tab)
            .render(title_area, buf);

        let content_block = Block::bordered();
        
        self.tabs.index(self.tab).render(content_block.inner(tab_area), buf);

        content_block.render(tab_area, buf);
    }
}