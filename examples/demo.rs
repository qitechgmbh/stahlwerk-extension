use std::{time::{Duration, Instant}, vec};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}
};

use beas_bsl::{ClientConfig};
use stahlwerk_extension::ff01::{Entry, ProxyClient, ProxyTransactionError};

use stahlwerk_extension::ff01::Request;
use stahlwerk_extension::ff01::Response;

pub fn main() -> std::io::Result<()>
{
    let config = ClientConfig::from_file("config.json").expect("Where config?");
    
    let proxy = ProxyClient::new(config).expect("Why no client?");

    let mut app = App {
        exit: Default::default(),
        state: Default::default(),
        entry: Default::default(),
        pending: Default::default(),
        machine_running: Default::default(),
        machine_counter: Default::default(),
        last_request_ts: Instant::now(),
        proxy,
    };

    ratatui::run(|terminal| app.run(terminal))
}

#[derive(Debug)]
pub struct App
{
    exit:    bool,

    state:   u32,

    entry:   Option<Entry>,
    pending: bool,

    // machine
    machine_running: bool,
    machine_counter: u32,

    last_request_ts: Instant,

    proxy: ProxyClient,
}

impl App 
{
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> 
    {
        while !self.exit
        {
            self.handle_events()?;
            self.update(terminal)?;
            terminal.draw(|frame| self.draw(frame))?;
            std::thread::sleep(Duration::from_millis(50));
        }
        
        Ok(())
    }

    fn handle_response(&mut self, response: Response)  {
        use Response::*;

        match &self.state
        {
            // get next entry
            0 => {
                let entry = match response {
                    GetNextEntry(v) => v,
                    _ => panic!("Tag Mismatch"),
                };

                let Some(entry) = entry else { return; };

                self.entry = Some(entry);
                self.machine_running = true;
                self.state = 1;
            }
            1 => {
                let entry = self.entry.as_mut().expect("Must be valid!");

                let scrap_quantity = match response {
                    GetScrapQuantity(v) => v.unwrap_or(0.0),
                    _ => panic!("Tag Mismatch"),
                };

                if scrap_quantity != 0.0 {
                    entry.scrap_quantity = scrap_quantity;
                    self.machine_running = false;
                    self.state = 2;
                }
            }
            2 => {
                _ = match response {
                    Finalize => {},
                    _ => panic!("Tag Mismatch"),
                };

                self.entry = None;
                self.machine_counter = 0;
                self.state = 0;
            }
            _ => {}
        }
    }

    fn update(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> 
    {   
        use ProxyTransactionError::Pending;

        let now = Instant::now();
        let req_timeout = Duration::from_millis(2000);

        if now.duration_since(self.last_request_ts) < req_timeout {
            return Ok(());
        }

        self.pending = true;
        terminal.draw(|frame| self.draw(frame))?;

        let request = match self.state {
            0 => Request::GetNextEntry,
            1 => Request::GetScrapQuantity(self.entry.as_ref().unwrap()),
            2 => Request::Finalize(&self.entry.as_ref().unwrap(), self.machine_counter),
            _ => panic!("Invalid State reached"),
        };

        self.proxy.queue_request(request).expect("Shouldn't fail");

        let response = loop {
            match self.proxy.poll_response() {
                Ok(v)  => break v,
                Err(e) => match e {
                    Pending => continue,
                    _ => panic!("Error while handling Transaction: {:?}", e),
                },
            }
        };

        self.last_request_ts = Instant::now();
        self.pending = false;

        self.handle_response(response);

        terminal.draw(|frame| self.draw(frame))?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> std::io::Result<()> 
    {
        if !event::poll(Duration::from_millis(1))? {
            return Ok(());
        }

        match event::read()? 
        {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) 
    {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(' ') => {
                if self.state != 1 { return; }
                self.machine_counter += 1
            },
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn draw_machine(&self, area: Rect, buf: &mut Buffer)
    {
        let title = match self.machine_running 
        {
            false => Line::from(vec![" Machine: ".white(), "Idle ".red()]),
            true  => Line::from(vec![" Machine: ".white(), "Running ".green()]),
        };

        let lines = vec![
            Line::from(vec![" Counter: ".white(), self.machine_counter.to_string().yellow()]),
            // Line::from(vec![" Weight: ".white(), 0.0.to_string().yellow()]),
        ];

        let block = Block::bordered()
            .title(title.centered().white())
            .border_set(border::PLAIN)
            .white();

        let text = Text::from(lines);

        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }

    fn draw_entry(&self, area: Rect, buf: &mut Buffer)
    {
        let (title, lines) = match &self.entry
        {
            Some(entry) => 
            {
                let title = Line::from(vec![" Entry: ".white(), "Found ".green()]);

                let scrap_quantity = match self.pending {
                    true  => "Updating...".cyan(),
                    false => entry.scrap_quantity.to_string().yellow(),
                };

                let lines = vec![
                    Line::from(vec![" DocEntry: ".white(), entry.doc_entry.to_string().yellow()]),
                    Line::from(vec![" LineNumber: ".white(), entry.line_number.to_string().yellow()]),
                    Line::from(vec![" ScrapQuantity: ".white(), scrap_quantity]),
                    Line::from(vec![" ItemCode: ".white(), entry.item_code.to_string().yellow()]),
                    Line::from(vec![" WhsCode: ".white(), entry.whs_code.to_string().yellow()]),

                    Line::from(vec![" WeightBounds: ".white()]),
                    Line::from(vec![" - Min: ".white(), entry.weight_bounds.min.to_string().yellow()]),
                    Line::from(vec![" - Max: ".white(), entry.weight_bounds.max.to_string().yellow()]),
                    Line::from(vec![" - Desired: ".white(), entry.weight_bounds.desired.to_string().yellow()]),
                ];

                (title, lines)
            },
            None => {
                let title_value = match self.pending {
                    true  => "Updating... ".cyan(),
                    false => "Missing ".red(),
                };

                let title = Line::from(vec![" Entry: ".white(), title_value]);

                let null = "null".red();
                let lines = vec![
                    Line::from(vec![" DocEntry: ".white(), null.clone()]),
                    Line::from(vec![" LineNumber: ".white(), null.clone()]),
                    Line::from(vec![" ScrapQuantity: ".white(), null.clone()]),
                    Line::from(vec![" ItemCode: ".white(), null.clone()]),
                    Line::from(vec![" WhsCode: ".white(), null.clone()]),

                    Line::from(vec![" WeightBounds: ".white()]),
                    Line::from(vec![" - Min: ".white(), null.clone()]),
                    Line::from(vec![" - Max: ".white(), null.clone()]),
                    Line::from(vec![" - Desired: ".white(), null.clone()]),
                ];

                (title, lines)
            },
        };

        let block = Block::bordered()
            .title(title.centered().white())
            .border_set(border::PLAIN)
            .white();

        let text = Text::from(lines);

        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}

impl Widget for &App 
{
    fn render(self, area: Rect, buf: &mut Buffer)
    {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        let box_0 = rows[0];
        let box_1 = rows[1];

        self.draw_entry(box_0, buf);
        self.draw_machine(box_1, buf);
    }
}