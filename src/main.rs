use std::io::{Stdout, Write};

use crossterm::{cursor::MoveTo, terminal::size, ExecutableCommand};
use fastnoise_lite::FastNoiseLite;

const BRIGHTNESS: [u8; 5] = [ b' ', b'.', b'-', b'=', b'#'];

fn draw_noise(stdout: &mut Stdout, noise: &FastNoiseLite, offset: f32) {
    let size = size().unwrap();

    stdout.execute(MoveTo(0, 0)).unwrap();

    let mut buf = vec![b' '; (size.0 * size.1) as usize];

    for y in 0..size.1 {
        for x in 0..size.0 {
            let value = (noise.get_noise_2d(x as f32, y as f32 + offset) + 1.0) / 2.0;
            let index = (value * BRIGHTNESS.len() as f32) as usize;

            buf[(y * size.0 + x) as usize] = BRIGHTNESS[index];
        }
    }

    stdout.write_all(&buf).unwrap();
}

fn main() {
    let mut noise = FastNoiseLite::new();
    noise.set_noise_type(Some(fastnoise_lite::NoiseType::OpenSimplex2S));

    let mut stdout = std::io::stdout();

    crossterm::terminal::enable_raw_mode().unwrap();

    let mut frame_times = Vec::new();

    for x in 0..100 {
        let start = std::time::Instant::now();

        draw_noise(&mut stdout, &noise, x as f32);

        let elapsed = start.elapsed();
        frame_times.push(elapsed);

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    crossterm::terminal::disable_raw_mode().unwrap();

    println!("Average frame time: {:?}", frame_times.iter().sum::<std::time::Duration>() / frame_times.len() as u32);
}
