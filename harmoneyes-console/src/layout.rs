use std::ops::Index;

use ratatui::{buffer::Buffer, layout::{Layout, Rect}, text::Line, widgets::{Bar, BarChart, BarGroup, Block, Tabs, Widget}};

#[derive(Default)]
pub struct AppView {
    tabs: Vec<Box<dyn Tab>>,
    tab: usize
}

impl AppView {
    pub fn new() -> Self {
        AppView {
            tabs: vec![Box::new(OverviewTab {}), Box::new(DeviceTab { name: "Device One".to_string() })],
            ..Default::default()
        }
    }
}

impl Widget for &AppView {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
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






pub trait Tab {
    fn title(&self) -> String;

    fn render(&self, area: Rect, buf: &mut Buffer);
}




#[derive(Default)]
pub struct OverviewTab {

}

impl Tab for OverviewTab {
    fn title(&self) -> String {
        "Overview".into()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::Constraint::{Length, Fill};

        let layout = Layout::horizontal([Length(32), Fill(1)]);
        let [battery_area, _other_area] = layout.areas(area);

        let block = Block::bordered()
            .title("Batteries");

        BarChart::default()
            .bar_width(14)
            .max(100)
            .data(BarGroup::default()
                .label(Line::from("Devices").centered())
                .bars(&[
                    Bar::default()
                        .label("One".into())
                        .value(87),
                    Bar::default()
                        .label("Two".into())
                        .value(45)
                ])
            )
            .block(block)
            .render(battery_area, buf);
    }
}







#[derive(Default)]
pub struct DeviceTab {
    pub name: String
}

impl Tab for DeviceTab {
    fn title(&self) -> String {
        self.name.clone()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Line::from("Device Tab").render(area, buf);
    }
}