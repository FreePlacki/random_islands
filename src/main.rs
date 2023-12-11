use rand::prelude::*;
use raylib::prelude::*;

#[derive(Debug)]
struct Grid {
    gradients: Vec<Vec<Vector2>>,
    size: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize, size: usize) -> Self {
        Self {
            gradients: vec![vec![rvec2(0.0, 0.0); width]; height],
            size,
        }
    }

    fn random_gradient_at(&mut self, xi: usize, yi: usize) -> Vector2 {
        let g = &mut self.gradients[yi][xi];

        if g.x == 0.0 && g.y == 0.0 {
            let x: f32 = random();
            g.y = (1.0 - x * x).sqrt();
            g.x = x;
        }

        *g
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

fn dot_gradient(xi: usize, yi: usize, v: Vector2, grid: &mut Grid) -> f32 {
    let grad = grid.random_gradient_at(xi, yi);
    let dist = v - rvec2(xi as f32, yi as f32);
    // let dist = dist.normalized();

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

fn get_color(value: f32) -> Color {
    let v = (value * 255.0) as u8;
    Color::new(v, v, v, 255)
}

fn main() {
    const WIDTH: i32 = 800;
    const HEIGHT: i32 = 800;
    const SIZE: usize = 20;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Perlin noise")
        .build();
    let mut grid = Grid::new(WIDTH as usize, HEIGHT as usize, SIZE);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                let noise = perlin(
                    rvec2(j as f32 / SIZE as f32, i as f32 / SIZE as f32),
                    &mut grid,
                );
                d.draw_pixel(j, i, get_color(noise));
            }
        }
    }
}

