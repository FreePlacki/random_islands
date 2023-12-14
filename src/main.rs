use std::f32::consts::PI;

use rand::prelude::*;
use raylib::{
    ffi::{GetMouseX, GetMouseY, IsKeyPressed, SetTargetFPS, IsKeyDown},
    prelude::*,
};

#[derive(Debug)]
struct Grid {
    pub gradients: Vec<Vec<Vector2>>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut gradients = vec![vec![rvec2(0.0, 0.0); width]; height];
        for i in 0..gradients.len() {
            for j in 0..gradients[0].len() {
                let x: f32 = map_to(random(), 0.0, 1.0, 0.0, 2.0 * PI);
                gradients[i][j].x = x.sin();
                gradients[i][j].y = x.cos();
            }
        }
        Self { gradients }
    }

    fn make_noise(
        &mut self,
        width: i32,
        height: i32,
        pixel: f32,
        octaves: i32,
        freq: f32,
        ampl: f32,
    ) -> Vec<Vec<f32>> {
        let mut res = vec![vec![]; self.gradients.len()];
        for i in 0..self.gradients.len() {
            for j in 0..self.gradients[0].len() {
                let mut noise = 0.0;
                let mut max_value = 0.0;
                for k in 0..octaves {
                    let f = freq * (2_f32).powi(k);
                    let f = f.min(0.9);
                    let a = ampl * (2_f32).powi(-k);

                    noise += perlin(rvec2(j as f32 * f, i as f32 * f), self) * a;
                    max_value += a;
                }

                let dist = ((height / 2 - i as i32).pow(2) + (width / 2 - j as i32).pow(2)) as f32;
                // let sc = 2.0_f32.powf(-dist * 1e-5 * pixel.powi(2));
                let sc = (2.0 * dist * 2.0_f32.powf(-dist * 1e-5 * pixel.powi(2))).powi(2);
                // let sc = 1.0 / (dist * 2e-5 * pixel as f32);
                // let sc = if sc > 1.0 {
                //     1.0
                // } else {
                //     sc
                // };

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

fn normalise(values: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let max = values
        .iter()
        .flat_map(|v| v.iter())
        .cloned()
        .fold(0.0, f32::max);

    values
        .iter()
        .map(|v| v.iter().map(|e| map_to(*e, 0.0, max, 0.0, 1.0)).collect())
        .collect()
}

fn map_to(x: f32, from1: f32, to1: f32, from2: f32, to2: f32) -> f32 {
    // [from1, to1] -> [from2, to2]
    (x - from1) / (to1 - from1) * (to2 - from2) + from2
}

fn dot_gradient(xi: usize, yi: usize, v: Vector2, grid: &mut Grid) -> f32 {
    let grad = grid.gradients[yi][xi];
    let dist = v - rvec2(xi as f32, yi as f32);

    grad.dot(dist)
}

fn perlin(v: Vector2, grid: &mut Grid) -> f32 {
    // cell coordinates
    let x0 = v.x.floor() as usize;
    let x1 = x0 + 1;
    let y0 = v.y.floor() as usize;
    let y1 = y0 + 1;

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

fn calc_shadows(heights: &Vec<Vec<f32>>, light: Vector3) -> Vec<Vec<f32>> {
    // idea: shadows[y][x] = 0.1 / height
    let mut shadows = vec![vec![0.0; heights[0].len()]; heights.len()];

    for (y, row) in heights.iter().enumerate() {
        for (x, height) in row.iter().enumerate() {
            // x/y = lightX/lightY; x^2 + y^2 = 1
            // BUG: sign issue

            let step = light - Vector3::new(x as f32, y as f32, *height);
            // let step = step.normalized() * (1.0 as f32);
            let step = step * (1e-3_f32);

            let mut curr = Vector3 {
                x: x as f32,
                y: y as f32,
                z: *height,
            };

            while curr.z <= 1.0 {
                curr += step;
                let x_ind = curr.x.floor() as usize;
                let y_ind = curr.y.floor() as usize;

                if x_ind >= heights[0].len() || y_ind >= heights.len() {
                    break;
                }

                if curr.z < heights[y_ind][x_ind] {
                    // shadows on water
                    if curr.z > 110.0 / 255.0 {
                        shadows[y][x] = 1.0;
                        break;
                    }
                }
            }
        }
    }

    shadows
}

fn get_color(height: f32, shadow: f32) -> Color {
    let v = map_to(height, 0.0, 1.0, 0.0, 255.0) as u8;
    let blue = Color::from_hex("448285").unwrap();
    let yell = Color::from_hex("fabd2f").unwrap();
    let gree = Color::from_hex("b8bb26").unwrap();
    let dgre = Color::from_hex("98971a").unwrap();
    let gray = Color::from_hex("928374").unwrap();
    let whit = Color::from_hex("fbf1c7").unwrap();
    let col = match v {
        0..=110 => blue,
        111..=130 => yell,
        131..=150 => gree,
        151..=180 => dgre,
        181..=210 => gray,
        _ => whit,
    };
    apply_shadow(col, shadow)
    // Color::new(v, v, v, 255)
}

fn apply_shadow(color: Color, shadow: f32) -> Color {
    let r = (color.r as f32 * (3.0_f32).powf(-shadow * 0.4)) as u8; // little more red
    let g = (color.g as f32 * (3.0_f32).powf(-shadow * 0.5)) as u8;
    let b = (color.b as f32 * (3.0_f32).powf(-shadow * 0.5)) as u8;
    Color::new(r, g, b, 255)
}

fn main() {
    unsafe { SetTargetFPS(30) };
    // let screen_w = GetScreenWidth() as usize;
    // let screen_w = GetScreenHeight() as usize;

    const PIXEL: usize = 4;
    const WIDTH: usize = 1920 / PIXEL;
    const HEIGHT: usize = 1080 / PIXEL;
    const SCALE: f32 = 0.01 * PIXEL as f32;

    let mut grid = Grid::new(WIDTH, HEIGHT);
    let heights = grid.make_noise(WIDTH as i32, HEIGHT as i32, PIXEL as f32, 10, SCALE, 1.0);
    let heights = normalise(heights);

    let (mut rl, thread) = raylib::init()
        .size((WIDTH * PIXEL) as i32, (HEIGHT * PIXEL) as i32)
        .title("Perlin noise")
        .build();

    let mut light_z = 2.5;
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        unsafe {
            if IsKeyDown(KeyboardKey::KEY_UP as i32) {
                light_z += 0.1;
            }
            if IsKeyDown(KeyboardKey::KEY_DOWN as i32) {
                light_z = 1.1_f32.max(light_z - 0.1);
            }
        }
        let light = unsafe {
            Vector3 {
                x: GetMouseX() as f32 / PIXEL as f32,
                y: GetMouseY() as f32 / PIXEL as f32,
                z: light_z,
            }
        };
        // let light = Vector3::new(1000.0, 1000.0, 2.5);
        let shadows = calc_shadows(&heights, light);

        let mut i = 0;
        while i < HEIGHT * PIXEL {
            let mut j = 0;
            while j < WIDTH * PIXEL {
                let col = get_color(heights[i / PIXEL][j / PIXEL], shadows[i / PIXEL][j / PIXEL]);
                for m in 0..PIXEL {
                    for n in 0..PIXEL {
                        d.draw_pixel((j + m) as i32, (i + n) as i32, col);
                    }
                }
                j += PIXEL;
            }
            i += PIXEL;
        }
    }
}
