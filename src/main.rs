use crossterm::{
    cursor,
    event::{self, Event},
    execute, queue,
    style::{self, Color, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::{rngs::ThreadRng, Rng};
use std::{
    io::{self, Stdout, Write},
    time::Duration,
};

// NOTICE: config
const SPEED: i32 = 1;
const P: f64 = 0.05;
const CHARS_MIN_LEN: usize = 10;

#[derive(Debug, Clone)]
struct Rain {
    x: u16,
    y: i32,
    chars: Vec<u8>,
    index: usize,
}

impl Rain {
    fn new(x: u16, h: i32, random: &mut ThreadRng) -> Self {
        let len: usize = random.gen_range(0..(h as usize / 5)) + CHARS_MIN_LEN;
        let mut chars = Vec::with_capacity(len);
        for _ in 0..len {
            chars.push(random.gen_range(33..=126));
        }
        Self {
            x,
            y: 1,
            chars,
            index: 0,
        }
    }

    fn drop(&mut self, h: i32) -> bool {
        if self.y + SPEED < h {
            self.y += SPEED;
        } else {
            if self.index >= self.chars.len() {
                return false;
            }
            let offset = (SPEED - (h - 1 - self.y)) as usize;
            self.y = h - 1;
            self.index += offset;
        }

        true
    }

    fn draw(&mut self, stdout: &mut Stdout) {
        let len = self.chars.len();
        if self.index < len {
            let mut y = self.y as u16;
            let per_rb: u8 = 255 / self.chars.len() as u8;
            let mut rb = 255 - (per_rb * self.index as u8);
            for i in self.index..self.chars.len() {
                let green = Color::Rgb {
                    r: (rb),
                    g: (255),
                    b: (rb),
                };
                rb -= per_rb;
                queue!(
                    stdout,
                    SetForegroundColor(green),
                    // SetBackgroundColor(Color::Rgb {
                    //     r: (0),
                    //     g: (rb),
                    //     b: (0)
                    // }),
                    cursor::MoveTo(self.x, y),
                    style::Print(self.chars[i] as char),
                    style::ResetColor,
                )
                .unwrap();
                if y == 0 {
                    break;
                }
                y -= 1;
            }
        }

        // Delete old tail character of rain
        queue!(stdout, style::ResetColor).unwrap();
        if self.y as usize >= len {
            let y = self.y - (std::cmp::max(len, self.index) - self.index) as i32;
            for i in 0..SPEED {
                if y - i < 0 {
                    break;
                }
                queue!(
                    stdout,
                    cursor::MoveTo(self.x, (y - i) as u16),
                    style::Print(" ")
                )
                .unwrap();
            }
        }
    }
}

struct App {
    h: i32,
    stdout: Stdout,
    rains: Vec<Option<Rain>>,
    random: ThreadRng,
}

impl App {
    fn new() -> Self {
        let (w, h) = terminal::size().unwrap();
        let rains = vec![None; w as usize];
        Self {
            h: h as i32,
            stdout: io::stdout(),
            rains,
            random: rand::thread_rng(),
        }
    }

    fn init(&mut self) {
        execute!(self.stdout, terminal::EnterAlternateScreen).unwrap();
        terminal::enable_raw_mode().unwrap();

        execute!(
            self.stdout,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
        )
        .unwrap();
    }

    fn clear(&mut self) {
        execute!(
            self.stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )
        .unwrap();
        terminal::disable_raw_mode().unwrap();
    }

    fn draw_update_rains(&mut self) {
        for i in 0..self.rains.len() {
            if let Some(ref mut rain) = self.rains[i] {
                if rain.drop(self.h) {
                    rain.draw(&mut self.stdout)
                } else {
                    self.rains[i] = None;
                }
            } else {
                if self.random.gen_bool(P) {
                    self.rains[i] = Some(Rain::new(i as u16, self.h, &mut self.random));
                }
            }
        }
        self.stdout.flush().unwrap();
    }

    fn user_input(&mut self) -> bool {
        if event::poll(Duration::from_millis(50)).unwrap() {
            match event::read().unwrap() {
                Event::Key(_) => return true,
                Event::Resize(w, h) => {
                    queue!(self.stdout, terminal::Clear(ClearType::All)).unwrap();
                    self.h = h as i32;
                    self.rains = vec![None; w as usize];
                }
                _ => {}
            }
        }
        false
    }

    fn main_loop(&mut self) {
        while !self.user_input() {
            self.draw_update_rains();
        }
    }
}

fn main() {
    let mut app = App::new();
    app.init();
    app.main_loop();
    app.clear();
}
