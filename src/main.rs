use std::f32::consts::PI;

use rand::prelude::*;
use raylib::prelude::*;

#[derive(Debug)]
struct Grid {
    pub gradients: Vec<Vec<Vector2>>,
    size: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize, size: usize) -> Self {
        let mut gradients = vec![vec![rvec2(0.0, 0.0); width / size]; height / size];
        for i in 0..gradients.len() {
            for j in 0..gradients[0].len() {
                let x: f32 = map_to(random(), 0.0, 1.0, 0.0, 2.0 * PI);
                gradients[i][j].x = x.sin();
                gradients[i][j].y = x.cos();
            }
        }
        Self { gradients, size }
    }

    fn make_noise(&mut self, width: i32, height: i32, octaves: i32, freq: f32, ampl: f32) -> Vec<Vec<f32>> {
        let mut res = vec![vec![]; self.gradients.len()];
        for i in 0..self.gradients.len() {
            for j in 0..self.gradients[0].len() {
                let mut noise = 0.0;
                let mut max_value = 0.0;
                for k in 0..octaves {
                    let f = freq * (2 as f32).powi(k);
                    let f = f.min(0.9);
                    let a = ampl * (2 as f32).powi(-k);

                    noise += perlin(rvec2(j as f32 * f, i as f32 * f), self) * a;
                    max_value += a;
                }
                let dist = ((height / 2 - i as i32).pow(2) + (width / 2 - j as i32).pow(2)) as f32;
                let sc = (2.0 as f32).powf(-dist * 1e-5);
                let noise = noise / max_value * sc;
                res[i].push(noise);
            }
        }

        res
    }
}

fn interpolate(a0: f32, a1: f32, w: f32) -> f32 {
    if w < 0.0 {
        a0
    } else if w > 1.0 {
        a1
    } else {
        // (a1 - a0) * w + a0
        // (a1 - a0) * (3.0 - w * 2.0) * w * w + a0
        (a1 - a0) * ((w * (w * 6.0 - 15.0) + 10.0) * w * w * w) + a0
    }
}

fn map_to(x: f32, from1: f32, to1: f32, from2: f32, to2: f32) -> f32 {
    // [from1, to1] -> [from2, to2]
    (x - from1) / (to1 - from1) * (to2 - from2) + from2
}

fn dot_gradient(xi: usize, yi: usize, v: Vector2, grid: &mut Grid) -> f32 {
    let grad = grid.gradients[yi / grid.size][xi / grid.size];
    let dist = v - rvec2(xi as f32, yi as f32);
    // let dist = dist.normalized();

    grad.dot(dist)
}

fn perlin(v: Vector2, grid: &mut Grid) -> f32 {
    // cell coordinates
    let x0 = v.x.floor() as usize;
    let x1 = x0 + grid.size;
    let y0 = v.y.floor() as usize;
    let y1 = y0 + grid.size;

    // interp. weights
    let sx = v.x - x0 as f32;
    let sy = v.y - y0 as f32;

    let n0 = dot_gradient(x0, y0, v, grid);
    let n1 = dot_gradient(x1, y0, v, grid);
    let ix0 = interpolate(n0, n1, sx);

    let n0 = dot_gradient(x0, y1, v, grid);
    let n1 = dot_gradient(x1, y1, v, grid);
    let ix1 = interpolate(n0, n1, sx);

    // in range [0, 1]
    interpolate(ix0, ix1, sy) * 0.5 + 0.5
}

fn get_color(value: f32) -> Color {
    let v = map_to(value, 0.0, 1.0, 0.0, 255.0) as u8;
    let blue = Color::from_hex("448285").unwrap();
    let yell = Color::from_hex("fabd2f").unwrap();
    let gree = Color::from_hex("b8bb26").unwrap();
    let dgre = Color::from_hex("98971a").unwrap();
    let gray = Color::from_hex("928374").unwrap();
    match v {
        0..=90 => blue,
        91..=100 => yell,
        101..=120 => gree,
        121..=150 => dgre,
        _ => gray,
    }
    // Color::new(v, v, v, 255)
}

fn main() {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 800;
    const SIZE: usize = 1;
    const SCALE: f32 = 0.01;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Perlin noise")
        .build();
    let mut grid = Grid::new(WIDTH, HEIGHT, SIZE);
    let noises = grid.make_noise(WIDTH as i32, HEIGHT as i32, 10, SCALE, 1.0);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                d.draw_pixel(j as i32, i as i32, get_color(noises[i][j]));
            }
        }
    }
}
