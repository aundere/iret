use std::{io::{Stdout, Write}};

use crossterm::{cursor::MoveTo, terminal::size, ExecutableCommand};
use fastnoise_lite::FastNoiseLite;

const BRIGHTNESS: [u8; 4] = [ b' ', b'.', b'o', b'#'];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32
}

struct Generator {
    noise: FastNoiseLite
}

struct Chunk {
    pos: Pos,
    data: [u8; 128 * 128]
}

impl Chunk {

    fn world_pos(&self) -> Pos {
        Pos {
            x: self.pos.x * 128,
            y: self.pos.y * 128
        }
    }
}

struct World {
    generator: Generator,
    chunks: Vec<Chunk>
}

impl Generator {

    fn new(seed: i32) -> Self {
        let mut noise = FastNoiseLite::new();
        noise.set_seed(Some(seed));
        noise.set_noise_type(Some(fastnoise_lite::NoiseType::OpenSimplex2S));
        noise.set_frequency(Some(0.02));

        Generator { noise }
    }

    fn generate_chunk(&self, pos: Pos) -> Chunk {
        let mut data = [0u8; 128 * 128];

        for y in 0..128 {
            for x in 0..128 {
                let noise_x = (pos.x * 128 + x) as f32;
                let noise_y = (pos.y * 128 + y) as f32;

                let value = self.noise.get_noise_2d(noise_x, noise_y);
                let index = ((value + 1.0) / 2.0 * BRIGHTNESS.len() as f32) as usize;

                data[(y * 128 + x) as usize] = BRIGHTNESS[index];
            }
        }

        Chunk { pos, data }
    }
}

impl World {

    pub fn new(seed: i32) -> Self {
        let generator = Generator::new(seed);
        World { generator, chunks: Vec::new() }
    }

    pub fn generate_chunk(&mut self, pos: Pos) {
        let chunk = self.generator.generate_chunk(pos);
        self.chunks.push(chunk);
    }

    pub fn acquire_chunk(&mut self, pos: Pos) -> &Chunk {
        let has_chunk = self.chunks.iter().any(|c| c.pos == pos);

        if !has_chunk {
            self.generate_chunk(pos);
        }

        self.chunks.iter().find(|c| c.pos == pos).expect("Chunk not found")
    }
}

fn render_chunk(stdout: &mut Stdout, camera_pos: Pos, chunk: &Chunk) {
    let screen_size = size().unwrap();
    let chunk_pos = chunk.world_pos();

    let (render_from, render_to) = (
        Pos { x: (camera_pos.x - chunk_pos.x).clamp(0, 128), y: (camera_pos.y - chunk_pos.y).clamp(0, 128) },
        Pos { x: (camera_pos.x - chunk_pos.x + screen_size.0 as i32).clamp(0, 128), y: (camera_pos.y - chunk_pos.y + screen_size.1 as i32).clamp(0, 128) }
    );

    let cursor_pos = Pos {
        x: (chunk_pos.x - camera_pos.x).clamp(0, screen_size.0 as i32 - 1),
        y: (chunk_pos.y - camera_pos.y).clamp(0, screen_size.1 as i32 - 1),
    };

    for y in render_from.y..render_to.y {
        let current_y = y - render_from.y;
        stdout.execute(MoveTo(cursor_pos.x as u16, (cursor_pos.y + current_y) as u16)).unwrap();

        for x in render_from.x..render_to.x {
            let index = (y * 128 + x) as usize;
            let char = chunk.data[index];

            stdout.write_all(&[char]).unwrap();
        }
    }
}

fn render_chunks_on_screen(stdout: &mut Stdout, world: &mut World, camera_pos: Pos) {
    let screen_size = size().unwrap();

    let first_chunk_pos = Pos {
        x: (camera_pos.x as f32 / 128.0).floor() as i32,
        y: (camera_pos.y as f32 / 128.0).floor() as i32
    };

    let last_chunk_pos = Pos {
        x: ((camera_pos.x as f32 + screen_size.0 as f32) / 128.0).floor() as i32,
        y: ((camera_pos.y as f32 + screen_size.1 as f32) / 128.0).floor() as i32,
    };

    for y in first_chunk_pos.y..=last_chunk_pos.y {
        for x in first_chunk_pos.x..=last_chunk_pos.x {
            let chunk_pos = Pos { x, y };
            let chunk = world.acquire_chunk(chunk_pos);
            render_chunk(stdout, camera_pos, chunk);
        }
    }
}

fn main() {
    let random_seed = rand::random::<i32>();

    let mut world = World::new(random_seed);
    let mut stdout = std::io::stdout();

    let mut camera_pos = Pos { x: 0, y: 0 };

    crossterm::terminal::enable_raw_mode().unwrap();

    loop {
        let key_event = crossterm::event::read().unwrap();

        if let crossterm::event::Event::Key(key) = key_event {
            if key.code == crossterm::event::KeyCode::Esc {
                break;
            }

            if key.code == crossterm::event::KeyCode::Char('w') {
                camera_pos.y -= 1;
            } else if key.code == crossterm::event::KeyCode::Char('s') {
                camera_pos.y += 1;
            } else if key.code == crossterm::event::KeyCode::Char('a') {
                camera_pos.x -= 1;
            } else if key.code == crossterm::event::KeyCode::Char('d') {
                camera_pos.x += 1;
            }
        };

        render_chunks_on_screen(&mut stdout, &mut world, camera_pos);
    }

    crossterm::terminal::disable_raw_mode().unwrap();
}
