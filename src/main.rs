use crossterm::{cursor, event, execute, terminal};
use std::time::{Duration, Instant};
use terminal_donut_rs::{render_scene, scene};

fn main() {
    terminal::enable_raw_mode().unwrap();
    execute!(
        std::io::stdout(),
        terminal::EnterAlternateScreen,
        cursor::Hide
    )
    .unwrap();

    let start_time = Instant::now();

    loop {
        if event::poll(Duration::from_millis(0)).unwrap() {
            if let event::Event::Key(event::KeyEvent { code: _, .. }) = event::read().unwrap() {
                break;
            }
        }
        let (screen_width, screen_height) = crossterm::terminal::size().unwrap();
        let time = (Instant::now() - start_time).as_secs_f32();
        let scene = scene(time);
        let buffer = render_scene(
            &scene,
            screen_width as usize - 1, // -1 to avoid automatic line breaks
            screen_height as usize,
            0.5,
        );

        execute!(std::io::stdout(), cursor::MoveTo(0, 0)).unwrap();
        print!("{buffer}");
    }

    execute!(
        std::io::stdout(),
        terminal::LeaveAlternateScreen,
        cursor::Show
    )
    .unwrap();

    terminal::disable_raw_mode().unwrap();
}
