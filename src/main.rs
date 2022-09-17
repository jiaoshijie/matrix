use crossterm::{
    cursor,
    event::{self, Event},
    execute, queue,
    style::{self, Color, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::{rngs::ThreadRng, Rng};
use std::{
    io::{self, Stdout, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

// NOTICE: config
const SPEED: i32 = 5;
const P: f64 = 0.05;
const DURATION_TIME: Duration = Duration::from_millis(75);
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
            y: 0,
            chars,
            index: 0,
        }
    }

    fn drop(&mut self, h: i32) {
        if self.y + SPEED < h {
            self.y += SPEED;
        } else {
            self.y = h - 1;
            self.index += SPEED as usize;
        }
    }

    fn draw(&mut self, stdout: &mut Stdout) -> bool {
        let len = self.chars.len();
        if self.index >= len {
            return false;
        }

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
                SetBackgroundColor(Color::Rgb {
                    r: (0),
                    g: (rb),
                    b: (0)
                }),
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

        true
    }
}

struct KeyChan {
    key_recv: Receiver<()>,
    key_send: Sender<()>,
}

struct ResizeChan {
    resize_recv: Receiver<(u16, u16)>,
    resize_send: Sender<(u16, u16)>,
}

struct App {
    h: i32,
    stdout: Stdout,
    rains: Vec<Option<Rain>>,
    random: ThreadRng,
    keychan: KeyChan,
    resizechan: ResizeChan,
}

impl App {
    fn new() -> Self {
        let (w, h) = terminal::size().unwrap();
        let rains = vec![None; w as usize];
        let (key_send, key_recv): (Sender<()>, Receiver<()>) = mpsc::channel();
        let (resize_send, resize_recv): (Sender<(u16, u16)>, Receiver<(u16, u16)>) =
            mpsc::channel();
        Self {
            h: h as i32,
            stdout: io::stdout(),
            rains,
            random: rand::thread_rng(),
            keychan: KeyChan { key_recv, key_send },
            resizechan: ResizeChan {
                resize_recv,
                resize_send,
            },
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

    fn update_rains(&mut self) {
        for i in 0..self.rains.len() {
            match &mut self.rains[i] {
                Some(rain) => rain.drop(self.h),
                None => {
                    if self.random.gen_bool(P) {
                        self.rains[i] = Some(Rain::new(i as u16, self.h, &mut self.random));
                    }
                }
            }
        }
    }

    fn draw(&mut self) {
        queue!(self.stdout, terminal::Clear(ClearType::All)).unwrap();
        for i in 0..self.rains.len() {
            match &mut self.rains[i] {
                Some(rain) => {
                    if !rain.draw(&mut self.stdout) {
                        self.rains[i] = None;
                    }
                }
                None => {}
            }
        }
        self.stdout.flush().unwrap();
    }

    fn appchan(&self) {
        let key_send = self.keychan.key_send.clone();
        let resize_send = self.resizechan.resize_send.clone();
        thread::spawn(move || loop {
            match event::read() {
                Ok(Event::Key(_)) => key_send.send(()).unwrap(),
                Ok(Event::Resize(w, h)) => {
                    let (rw, rh) = flush_resize_events((w, h));
                    resize_send.send((rw, rh)).unwrap()
                }
                Ok(_) => {}
                Err(_) => {}
            }
        });
    }

    fn main_loop(&mut self) {
        self.appchan();

        loop {
            // TODO: write a macro to make this smaller.
            match self.keychan.key_recv.try_recv() {
                Ok(_) => break,
                Err(_) => {}
            }
            match self.resizechan.resize_recv.try_recv() {
                Ok((w, h)) => {
                    self.h = h as i32;
                    self.rains = vec![None; w as usize];
                }
                Err(_) => {}
            }
            self.update_rains();
            self.draw();
            thread::sleep(DURATION_TIME);
        }
    }
}

fn main() {
    let mut app = App::new();
    app.init();
    app.main_loop();
    app.clear();
}

// Resize events can occur in batches.
// With a simple loop they can be flushed.
// This function will return last resize event.
fn flush_resize_events(first_resize: (u16, u16)) -> (u16, u16) {
    let mut last_resize = first_resize;
    while let Ok(true) = event::poll(Duration::from_millis(50)) {
        if let Ok(Event::Resize(x, y)) = event::read() {
            last_resize = (x, y);
        }
    }

    last_resize
}
