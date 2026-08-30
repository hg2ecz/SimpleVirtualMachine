use std::env;

use minifb::{Key, Window, WindowOptions};
use svm_rt::{
    machine::Machine,
    program::Program,
    video::{HEIGHT, WIDTH},
};

const INSTRUCTIONS_PER_FRAME: usize = 50_000;
const WINDOW_SCALE: usize = 2;

fn key_to_ascii(key: Key) -> Option<u8> {
    match key {
        Key::Escape => Some(27),
        Key::Enter => Some(13),
        Key::Space => Some(b' '),
        Key::A => Some(b'A'),
        Key::B => Some(b'B'),
        Key::C => Some(b'C'),
        Key::D => Some(b'D'),
        Key::E => Some(b'E'),
        Key::F => Some(b'F'),
        Key::G => Some(b'G'),
        Key::H => Some(b'H'),
        Key::I => Some(b'I'),
        Key::J => Some(b'J'),
        Key::K => Some(b'K'),
        Key::L => Some(b'L'),
        Key::M => Some(b'M'),
        Key::N => Some(b'N'),
        Key::O => Some(b'O'),
        Key::P => Some(b'P'),
        Key::Q => Some(b'Q'),
        Key::R => Some(b'R'),
        Key::S => Some(b'S'),
        Key::T => Some(b'T'),
        Key::U => Some(b'U'),
        Key::V => Some(b'V'),
        Key::W => Some(b'W'),
        Key::X => Some(b'X'),
        Key::Y => Some(b'Y'),
        Key::Z => Some(b'Z'),
        Key::Key0 => Some(b'0'),
        Key::Key1 => Some(b'1'),
        Key::Key2 => Some(b'2'),
        Key::Key3 => Some(b'3'),
        Key::Key4 => Some(b'4'),
        Key::Key5 => Some(b'5'),
        Key::Key6 => Some(b'6'),
        Key::Key7 => Some(b'7'),
        Key::Key8 => Some(b'8'),
        Key::Key9 => Some(b'9'),
        _ => None,
    }
}

fn spawn_console_input() -> std::sync::mpsc::Receiver<u8> {
    use std::io::Read;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut input = stdin.lock();
        let mut byte = [0u8; 1];
        loop {
            match input.read(&mut byte) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if tx.send(byte[0]).is_err() {
                        break;
                    }
                }
            }
        }
    });
    rx
}

struct RawModeGuard(bool);
impl RawModeGuard {
    fn new() -> Self {
        Self(crossterm::terminal::enable_raw_mode().is_ok())
    }
}
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.0 {
            let _ = crossterm::terminal::disable_raw_mode();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: svm-rt <program.svm|svs|sva|svf|svl|svr|svc|svb|svt>")?;
    let program = Program::load_file(path)?;

    let mut machine = Machine::new(program.cpu);
    machine.load_program(&program)?;
    let _raw_mode = RawModeGuard::new();
    let console_rx = spawn_console_input();

    let mut window = Window::new(
        &format!("SVM {}", program.cpu.name()),
        WIDTH * WINDOW_SCALE,
        HEIGHT * WINDOW_SCALE,
        WindowOptions::default(),
    )?;
    window.set_target_fps(60);

    let mut pixels = vec![0u32; WIDTH * HEIGHT];
    while window.is_open() {
        for byte in console_rx.try_iter() {
            machine.console_receive(byte);
        }
        let key = window.get_keys().into_iter().find_map(key_to_ascii);
        machine.set_key(key);

        for _ in 0..INSTRUCTIONS_PER_FRAME {
            if machine.halted() {
                break;
            }
            machine.step()?;
        }

        {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            while let Some(byte) = machine.take_console_tx() {
                out.write_all(&[byte])?;
            }
            out.flush()?;
        }

        machine.video_vsync();
        machine.render_argb(&mut pixels)?;
        window.update_with_buffer(&pixels, WIDTH, HEIGHT)?;

        if machine.halted() {
            break;
        }
    }
    Ok(())
}
