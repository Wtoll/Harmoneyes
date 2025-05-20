use std::{cell::OnceCell, collections::HashMap, pin::pin, time::Duration};

use crossterm::event::{EventStream, KeyCode, KeyModifiers};
use futures::{future::{join, select}, stream::FuturesUnordered, StreamExt};
use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Text}, widgets::{Block, Paragraph, Widget}, DefaultTerminal, Frame};
use tokio::{sync::Mutex, time::interval};
use tokio_serial::{SerialPort, SerialPortInfo, SerialPortType};
use tokio_util::bytes::BufMut;


#[tokio::main]
async fn main() {
    App::new().run().await;
}

struct App {
    terminal: OnceCell<Mutex<DefaultTerminal>>,
    tab: Mutex<usize>,
    connections: Mutex<HashMap<String, ConnectionHandler>>
}

impl App {
    fn new() -> Self {
        Self {
            terminal: OnceCell::new(),
            tab: Mutex::new(0),
            connections: Mutex::new(HashMap::new())
        }
    }

    pub async fn run(&mut self) {
        self.terminal.set(Mutex::new(ratatui::init())).expect("");

        let connections = pin!(self.handle_connections());
        let rendering = pin!(self.renderer());
        let events = pin!(self.handle_events());

        let _ = select(join(connections, rendering), events).await;

        ratatui::restore();
    }

    async fn handle_connections(&self) {
        self.find_connections().await;
    }

    async fn find_connections(&self) {
        let mut interval = interval(Duration::from_millis(250));

        loop {
            let ports = tokio_serial::available_ports().expect("");

            let device_ports: Vec<SerialPortInfo> = ports.into_iter().filter(|port| {
                if let SerialPortType::UsbPort(info) = &port.port_type {
                    return info.vid == harmoneyes_core::constants::USB_VENDOR_ID && info.pid == harmoneyes_core::constants::controller::USB_PRODUCT_ID;
                }
                return false;
            }).collect();

            // New scope to make clear where the mutex guard is dropped
            {
                let mut connections = self.connections.lock().await;
                for port in device_ports {
                    if !connections.contains_key(&port.port_name) {
                        connections.insert(port.port_name.clone(), ConnectionHandler::new(port));
                    }
                }
            }

            interval.tick().await;
        }
    }

    async fn renderer(&self) {
        let frequency = 60;
        let mut interval = interval(Duration::from_millis(1000 / frequency));

        let mut terminal = self.terminal.get().unwrap().lock().await;

        loop {
            let view = self.make_view().await;
            let _ = terminal.draw(|frame| {
                frame.render_widget(view, frame.area());
            });
            interval.tick().await;
        }
    }

    async fn make_view(&self) -> AppView {

        let connection_keys = {
            let mut connection_keys = Vec::new();

            let lock = self.connections.lock().await;

            for key in lock.keys() {
                connection_keys.push(key.clone());
            }

            connection_keys
        };

        AppView::new(connection_keys)
    }

    async fn tab_left(&self) {

    }

    async fn tab_right(&self) {

    }

    async fn handle_events(&self) {
        use crossterm::event::Event;

        let mut events = EventStream::new();

        loop {
            match events.next().await {
                Some(res) => match res {
                    Ok(e) => match e {
                        Event::FocusGained => {},
                        Event::FocusLost => {},
                        Event::Key(key_event) => {
                            if key_event.is_press() && key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == KeyCode::Char('c') {
                                break
                            }
                            if key_event.code == KeyCode::Left || key_event.code == KeyCode::Char('A') { self.tab_left().await; }
                            if key_event.code == KeyCode::Right || key_event.code == KeyCode::Char('D') { self.tab_right().await; }
                        },
                        Event::Mouse(mouse_event) => {},
                        Event::Paste(_) => {},
                        Event::Resize(_, _) => {},
                    },
                    Err(e) => {
                        
                    },
                },
                None => break,
            }
        }
    }
}



struct AppView {
    connection_keys: Vec<String>
}

impl AppView {
    fn new(connection_keys: Vec<String>) -> Self {
        Self {
            connection_keys
        }
    }
}

impl Widget for AppView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_block = Block::bordered();

        let lines: Vec<Line> = self.connection_keys.into_iter().map(|k| Line::from(k)).collect();

        let p = Paragraph::new(lines)
            .block(content_block);

        p.render(area, buf);
    }
}








struct ConnectionHandler {
    dropped: bool,
    port: Mutex<Option<Box<dyn SerialPort + 'static>>>
}

impl ConnectionHandler {
    pub fn new(port_info: SerialPortInfo) -> Self {
        match tokio_serial::new(port_info.port_name, 9600).open() {
            Ok(port) => Self {
                dropped: false,
                port: Mutex::new(Some(port))
            },
            _ => Self {
                dropped: true,
                port: Mutex::new(None)
            }
        }
    }

    pub async fn drive(&self) {
        if !self.dropped {
            
        }
    }
}

