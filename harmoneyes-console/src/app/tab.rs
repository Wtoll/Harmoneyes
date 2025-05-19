use ratatui::{buffer::Buffer, layout::{Layout, Rect}, text::Line, widgets::{Bar, BarChart, BarGroup, Block, Widget}};

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