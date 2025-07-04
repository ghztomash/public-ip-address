use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use public_ip_address::{lookup::LookupProvider, response::LookupResponse};
use ratatui::{
    prelude::*,
    symbols::Marker,
    widgets::{canvas::*, *},
};
use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};

#[cfg_attr(not(feature = "blocking"), tokio::main)]
#[maybe_async::maybe_async]
async fn main() -> io::Result<()> {
    App::run().await
}

struct App {
    x: f64,
    y: f64,
    scale: f64,
    tick_count: u64,
    geolocation: Option<LookupResponse>,
    marker: Marker,
    markers: Vec<Marker>,
}

impl App {
    fn new() -> App {
        App {
            x: 0.0,
            y: 0.0,
            scale: 0.25,
            tick_count: 0,
            geolocation: None,
            marker: Marker::Braille,
            markers: vec![
                Marker::Braille,
                Marker::Dot,
                Marker::Block,
                Marker::HalfBlock,
                Marker::Bar,
            ],
        }
    }

    #[maybe_async::maybe_async]
    pub async fn run() -> io::Result<()> {
        let mut terminal = init_terminal()?;
        let mut app = App::new();

        // Lookup the geolocation of the public IP address
        app.lookup().await;

        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);
        loop {
            let _ = terminal.draw(|frame| app.ui(frame));
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Enter => app.lookup().await,
                        KeyCode::Down | KeyCode::Char('j') => app.y -= 5.0 * app.scale,
                        KeyCode::Up | KeyCode::Char('k') => app.y += 5.0 * app.scale,
                        KeyCode::Right | KeyCode::Char('l') => app.x += 5.0 * app.scale,
                        KeyCode::Left | KeyCode::Char('h') => app.x -= 5.0 * app.scale,
                        KeyCode::Char('+') | KeyCode::Char('=') => app.scale *= 2.0,
                        KeyCode::Char('-') | KeyCode::Char('_') => app.scale /= 2.0,
                        KeyCode::Char('1') => app.marker = app.markers[0],
                        KeyCode::Char('2') => app.marker = app.markers[1],
                        KeyCode::Char('3') => app.marker = app.markers[2],
                        KeyCode::Char('4') => app.marker = app.markers[3],
                        KeyCode::Char('5') => app.marker = app.markers[4],
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                app.on_tick();
                last_tick = Instant::now();
            }
        }
        restore_terminal()
    }

    #[maybe_async::maybe_async]
    async fn lookup(&mut self) {
        self.geolocation = public_ip_address::perform_cached_lookup_with(
            vec![
                (LookupProvider::IpInfo, None),
                (LookupProvider::IpWhoIs, None),
                (LookupProvider::IpApiCo, None),
                (LookupProvider::IpLeak, None),
            ],
            None,
            Some(5),
            false,
        )
        .await
        .ok();
        if let Some(ref geo) = self.geolocation {
            self.x = geo.longitude.unwrap_or(0.0).round();
            self.y = geo.latitude.unwrap_or(0.0).round();
        }
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;
    }

    fn ui(&self, frame: &mut Frame) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]);
        let [map, right] = horizontal.areas(frame.area());

        frame.render_widget(self.map_canvas(), map);
        frame.render_widget(self.data_block(), right);
    }

    fn map_canvas(&self) -> impl Widget + '_ {
        // get the location of the public IP address
        let (ip, x, y, location) = match self.geolocation {
            Some(ref geo) => (
                geo.ip.to_string(),
                geo.longitude.unwrap_or(0.0),
                geo.latitude.unwrap_or(0.0),
                format!(
                    "{}, {}",
                    geo.city.as_deref().unwrap_or("unknown"),
                    geo.country.as_deref().unwrap_or("unknown")
                ),
            ),
            None => ("".to_string(), self.x, self.y, "unknown".to_string()),
        };

        Canvas::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" IP Location - Scale: {} ", self.scale)),
            )
            .marker(self.marker)
            .paint(move |ctx| {
                ctx.draw(&Map {
                    color: Color::Green,
                    resolution: MapResolution::High,
                });

                let text = ratatui::prelude::Line::from(vec![
                    Span::styled("X", Style::new().red().bold()),
                    Span::styled(format!(" <- {ip} ({location})"), Style::new().yellow()),
                ]);
                // geolocation
                ctx.print(x, y, text);
            })
            .x_bounds([self.x - 180.0 * self.scale, self.x + 180.0 * self.scale])
            .y_bounds([self.y - 126.0 * self.scale, self.y + 126.0 * self.scale])
    }

    fn data_block(&self) -> impl Widget + '_ {
        let data = match self.geolocation {
            Some(ref geo) => geo.to_string(),
            None => "No data available.".to_string(),
        };
        Paragraph::new(data)
            .block(Block::new().title(" Data ").borders(Borders::ALL))
            .style(Style::new().white())
            .wrap(Wrap { trim: true })
    }
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
