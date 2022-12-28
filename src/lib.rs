#![feature(new_uninit)]
#![feature(unchecked_math)]
use std::{alloc::Layout, mem::MaybeUninit};

use itertools::izip;
mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub fn lensq(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

impl core::ops::AddAssign for V2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl core::ops::Sub for V2 {
    type Output = V2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl core::ops::Add for V2 {
    type Output = V2;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl core::ops::Mul<V2> for f32 {
    type Output = V2;

    fn mul(self, rhs: V2) -> Self::Output {
        rhs * self
    }
}

impl core::ops::Mul<f32> for V2 {
    type Output = V2;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl core::ops::Mul for V2 {
    type Output = V2;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

type A<T> = Box<[T]>;

// assumes the array has been allocated to have N * 2 colors where N = T / dt = pi/2 * min_v / dt

#[wasm_bindgen]
pub struct World {
    boundary_radius: f32,
    ball_radius: f32,
    spawn: V2,
    dir: A<V2>,
    vel: A<f32>,
    pos: A<V2>,
    overlay_out: A<f32>,
    overlay_in: A<f32>,
    colors: A<u32>,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
}

impl RGBColor {
    pub const fn from_value(v: u32) -> Self {
        Self {
            r: (v >> 16) as u8,
            g: ((v >> 8) & 0xff) as u8,
            b: (v & 0xff) as u8,
        }
    }
}

#[wasm_bindgen]
impl RGBColor {
    pub fn number(&self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
    }
}

impl From<C3> for RGBColor {
    fn from(c: C3) -> Self {
        let r = (255.0 * c.r).floor() as u8;
        let g = (255.0 * c.g).floor() as u8;
        let b = (255.0 * c.b).floor() as u8;

        Self { r, g, b }
    }
}

#[derive(Clone, Copy)]
struct C3 {
    r: f32,
    g: f32,
    b: f32,
}

#[wasm_bindgen]
struct ColorCalc {
    color_indices: A<u32>,
}

#[wasm_bindgen]
impl ColorCalc {
    pub fn color_count(&self) -> usize {
        self.color_indices.len()
    }

    pub fn colors(&self) -> *const u32 {
        self.color_indices.as_ptr()
    }

    pub fn new(boundary_radius: f32, nballs: u32, dt: f32) -> Self {
        Self {
            color_indices: World::calculate_all_colors(boundary_radius, nballs, dt),
        }
    }
}

#[wasm_bindgen]
impl World {
    pub fn get_color_capacity(&self, dt: f32) -> usize {
        (self.vel.last().unwrap() * self.boundary_radius / dt).ceil() as usize
    }
    // pub fn calculate_all_colors(&mut self, dt: f32) -> Vec<u32> {

    // }

    fn calculate_all_colors(boundary_radius: f32, nballs: u32, dt: f32) -> A<u32> {
        let v_min = 100.0 / nballs as f32;

        let mut a = A::new_uninit_slice((v_min * boundary_radius / dt).ceil() as usize);
        const IN_BOUNCE_COL: C3 = C3 {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        };
        const OUT_BOUNCE_COL: C3 = C3 {
            r: 0.0,
            g: 0.7,
            b: 1.0,
        };

        let big_t = a.len() as f32 * dt;

        for (step, col_out) in a.iter_mut().enumerate() {
            let t = step as f32 * dt;
            let t_blue = big_t - t;
            let t_red = t;

            let blue_alpha = (-0.75 * t_blue).exp();
            let red_alpha = (-0.55 * t_red).exp();

            let final_col = OUT_BOUNCE_COL * blue_alpha + IN_BOUNCE_COL * red_alpha;

            col_out.write(RGBColor::from(final_col).number());
        }

        unsafe { a.assume_init() }
    }

    pub fn new(
        nballs: u32,
        boundary_radius: f32,
        ball_radius: f32,
        spawn_x: f32,
        spawn_y: f32,
        dt: f32,
    ) -> World {
        let spawn = V2 {
            x: spawn_x,
            y: spawn_y,
        };

        World {
            boundary_radius,
            ball_radius,
            colors: vec![0; nballs as usize].into_boxed_slice(),
            overlay_in: vec![100.0f32; nballs as usize].into_boxed_slice(),
            overlay_out: vec![100.0f32; nballs as usize].into_boxed_slice(),
            pos: vec![spawn; nballs as usize].into_boxed_slice(),
            vel: {
                let mut a = A::new_uninit_slice(nballs as usize);
                for (i, m) in a.iter_mut().enumerate() {
                    let v = 100.0 - (100.0 * i as f32 / nballs as f32);
                    m.write(v);
                }

                unsafe { a.assume_init() }
            },
            dir: {
                let mut a = A::new_uninit_slice(nballs as usize);
                for (i, m) in a.iter_mut().enumerate() {
                    let phi = 2.0 * core::f32::consts::PI * (i as f32 / nballs as f32);
                    m.write(V2 {
                        x: phi.cos(),
                        y: phi.sin(),
                    });
                }
                unsafe { a.assume_init() }
            },
            spawn,
        }
    }

    fn update_overlays(&mut self, dt: f32) {
        for i in self.overlay_out.iter_mut() {
            *i += dt;
        }

        for i in self.overlay_in.iter_mut() {
            *i += dt;
        }
    }

    fn update_positions(&mut self, dt: f32) {
        for (pos, (dir, vel)) in self
            .pos
            .iter_mut()
            .zip(self.dir.iter().copied().zip(self.vel.iter().copied()))
        {
            *pos += dir * vel * dt;
        }
    }

    fn border_check(&mut self) {
        for (pos, overlay_in, overlay_out, vel, dir) in izip!(
            self.pos.iter_mut(),
            self.overlay_in.iter_mut(),
            self.overlay_out.iter_mut(),
            self.vel.iter_mut(),
            self.dir.iter().copied(),
        ) {
            let dr = *pos - self.spawn;
            let lsq = dr.lensq();
            if lsq < 1.0 {
                *overlay_in = 0.0;
            }
            if lsq > self.boundary_radius * self.boundary_radius {
                *overlay_out = 0.0;
                let sign = vel.signum();
                *pos = self.spawn + (sign * self.boundary_radius) * dir;
                *vel *= -1.0;
            }
        }
    }

    pub fn simulate(&mut self, dt: f32) {
        self.update_overlays(dt);
        self.update_positions(dt);
        self.border_check();
    }

    pub fn x(&self, i: u32) -> f32 {
        self.pos[i as usize].x.floor()
    }

    pub fn y(&self, i: u32) -> f32 {
        self.pos[i as usize].y.floor()
    }

    pub fn color(&self, i: u32) -> u32 {
        self.colors[i as usize]
    }

    pub fn indices(&self) -> *const u32 {
        self.colors.as_ptr()
    }

    pub fn prepare_colors(&mut self, color_count: u32) {
        for (target_idx, pos) in izip!(self.colors.iter_mut(), self.pos.iter().copied()) {
            let rsq = (pos - self.spawn).lensq();

            let t = rsq.sqrt() / self.boundary_radius;

            let step = (t * (color_count - 1) as f32).floor() as u32;

            *target_idx = step;
        }
    }
}

fn addc(a: C3, b: C3, alpha: f32) -> C3 {
    let n = alpha;
    a * n + b * alpha
}

impl core::ops::Mul<f32> for C3 {
    type Output = C3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl core::ops::Add for C3 {
    type Output = C3;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}
