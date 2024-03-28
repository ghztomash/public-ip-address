use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::*, *},
};

use public_ip_address::{lookup::LookupProvider, LookupResponse};

use once_cell::sync::Lazy;

static PROVIDERS: Lazy<Vec<LookupProvider>> = Lazy::new(|| {
    vec![
        LookupProvider::IpInfo,
        LookupProvider::IpWhoIs,
        LookupProvider::IpApiCo,
    ]
});

fn main() -> io::Result<()> {
    App::run()
}

struct App {
    x: f64,
    y: f64,
    scale: f64,
    tick_count: u64,
    geolocation: Option<LookupResponse>,
    marker: Marker,
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
        }
    }

    pub fn run() -> io::Result<()> {
        let mut terminal = init_terminal()?;
        let mut app = App::new();

        // Lookup the geolocation of the public IP address
        app.lookup();

        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);
        loop {
            let _ = terminal.draw(|frame| app.ui(frame));
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Enter => app.lookup(),
                        KeyCode::Down | KeyCode::Char('j') => app.y -= 5.0 * app.scale,
                        KeyCode::Up | KeyCode::Char('k') => app.y += 5.0 * app.scale,
                        KeyCode::Right | KeyCode::Char('l') => app.x += 5.0 * app.scale,
                        KeyCode::Left | KeyCode::Char('h') => app.x -= 5.0 * app.scale,
                        KeyCode::Char('+') | KeyCode::Char('=') => app.scale *= 2.0,
                        KeyCode::Char('-') | KeyCode::Char('_') => app.scale /= 2.0,
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

    fn lookup(&mut self) {
        self.geolocation =
            public_ip_address::perform_cached_lookup_with_list(PROVIDERS.to_vec(), None).ok();
        if let Some(ref geo) = self.geolocation {
            self.x = geo.longitude.unwrap_or(0.0).round();
            self.y = geo.latitude.unwrap_or(0.0).round();
        }
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;
    }

    fn ui(&self, frame: &mut Frame) {
        frame.render_widget(self.map_canvas(), frame.size());
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

                // geolocation
                ctx.print(x, y, "X".red());
                ctx.print(
                    x + 4.0 * self.scale,
                    y,
                    format!("<- {} ({})", ip, location).yellow(),
                );
            })
            .x_bounds([self.x - 180.0 * self.scale, self.x + 180.0 * self.scale])
            .y_bounds([self.y - 90.0 * self.scale, self.y + 90.0 * self.scale])
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
