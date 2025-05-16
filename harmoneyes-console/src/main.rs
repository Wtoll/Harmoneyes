use app::App;

mod app;

fn main() {
    let mut terminal = ratatui::init();
    App::new().run(&mut terminal).unwrap();
    ratatui::restore();
}
