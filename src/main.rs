use glam::Vec2;
use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use rayon::prelude::*;
use std::time::Instant;

// Base simulation constants
const BASE_G: f32 = 100.0;        // base gravitational constant
const SOFTENING: f32 = 5.0;       // softening factor to prevent numerical instability
const BASE_DT: f32 = 0.008;       // base timestep
const NUM_BODIES: usize = 500;    // number of bodies
const WIDTH: usize = 3840;        // 4K resolution
const HEIGHT: usize = 2160;
const MIN_MASS: f32 = 1.0;
const MAX_MASS: f32 = 80.0;
const MAX_VELOCITY: f32 = 800.0;
const SPACE_SCALE: f32 = 1.0;
const CENTRAL_MASS: f32 = 2000.0;

#[derive(Clone)]
struct Body {
    pos: Vec2,
    vel: Vec2,
    mass: f32,
    color: u32,
}

impl Body {
    fn new(pos: Vec2, vel: Vec2, mass: f32) -> Self {
        // Color based on mass (blue for small, red for large)
        let t = (mass - MIN_MASS) / (MAX_MASS - MIN_MASS);
        let r = (t * 255.0) as u32;
        let g = ((1.0 - t * t) * 200.0) as u32;
        let b = ((1.0 - t) * 255.0) as u32;
        let color = (r << 16) | (g << 8) | b;
        
        Body { pos, vel, mass, color }
    }

    fn central() -> Self {
        Body::new(
            Vec2::ZERO,
            Vec2::ZERO,
            CENTRAL_MASS,
        )
    }

    fn random(g: f32) -> Self {
        let mut rng = rand::thread_rng();
        
        // Create denser orbital shells
        let shell = rng.gen_range(0..6);
        let base_distance = 150.0 * (shell + 1) as f32;
        let distance = base_distance + rng.gen_range(-30.0..30.0);
        
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let pos = Vec2::new(
            distance * angle.cos(),
            distance * angle.sin(),
        );
        
        // Calculate orbital velocity with some randomized eccentricity
        let orbit_speed = (g * CENTRAL_MASS / distance).sqrt() * rng.gen_range(0.7..1.4);
        let tangent = Vec2::new(-pos.y, pos.x).normalize();
        let outward = pos.normalize();
        let vel = (tangent + outward * rng.gen_range(-0.2..0.2)) * orbit_speed;

        Body::new(
            pos,
            vel,
            rng.gen_range(MIN_MASS..MAX_MASS),
        )
    }

    fn update(&mut self, force: Vec2, dt: f32) {
        let acc = force / self.mass;
        self.vel += acc * dt;
        
        // Dampen velocity if it exceeds maximum
        if self.vel.length() > MAX_VELOCITY {
            self.vel = self.vel.normalize() * MAX_VELOCITY;
        }
        
        self.pos += self.vel * dt;

        // Bounce off walls
        let bounds_x = (WIDTH as f32 / 2.0) * SPACE_SCALE;
        let bounds_y = (HEIGHT as f32 / 2.0) * SPACE_SCALE;
        
        if self.pos.x.abs() > bounds_x {
            self.vel.x *= -0.5;
            self.pos.x = self.pos.x.signum() * bounds_x;
        }
        if self.pos.y.abs() > bounds_y {
            self.vel.y *= -0.5;
            self.pos.y = self.pos.y.signum() * bounds_y;
        }
    }

    fn radius(&self) -> f32 {
        if self.mass == CENTRAL_MASS {
            25.0 // Fixed size for central body
        } else {
            (self.mass / MIN_MASS).sqrt() * 3.0
        }
    }
}

fn calculate_forces(bodies: &[Body], g: f32) -> Vec<Vec2> {
    bodies
        .par_iter()
        .map(|body1| {
            let mut force = Vec2::ZERO;

            for body2 in bodies {
                if std::ptr::eq(body1, body2) {
                    continue;
                }

                let r = body2.pos - body1.pos;
                let dist_sq = r.length_squared() + SOFTENING * SOFTENING;
                force += g * body1.mass * body2.mass * r.normalize() / dist_sq;
            }

            force
        })
        .collect()
}

fn draw_circle(buffer: &mut Vec<u32>, center: Vec2, radius: f32, color: u32, is_central: bool) {
    let x_center = (center.x / SPACE_SCALE + WIDTH as f32/2.0) as i32;
    let y_center = (center.y / SPACE_SCALE + HEIGHT as f32/2.0) as i32;
    let r = radius as i32;

    // Add glow effect for central body
    let glow_radius = if is_central { r * 2 } else { r + 2 };
    let r_squared = r * r;
    let glow_squared = glow_radius * glow_radius;

    for y in -glow_radius..=glow_radius {
        let y_offset = (y_center + y) as usize * WIDTH;
        let y_sq = y * y;
        
        for x in -glow_radius..=glow_radius {
            let dist_sq = x*x + y_sq;
            if dist_sq <= glow_squared {
                let px = x_center + x;
                
                if px >= 0 && px < WIDTH as i32 && (y_center + y) >= 0 && (y_center + y) < HEIGHT as i32 {
                    let idx = y_offset + px as usize;
                    if dist_sq <= r_squared {
                        buffer[idx] = color;
                    } else {
                        // Glow effect
                        let glow_intensity = 1.0 - (dist_sq as f32 / glow_squared as f32);
                        let r = ((color >> 16) & 0xFF) as f32 * glow_intensity;
                        let g = ((color >> 8) & 0xFF) as f32 * glow_intensity;
                        let b = (color & 0xFF) as f32 * glow_intensity;
                        buffer[idx] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                    }
                }
            }
        }
    }
}

fn main() {
    let mut window = Window::new(
        "N-Body Simulation (4K)",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    
    // Create bodies with central mass
    let mut bodies = Vec::with_capacity(NUM_BODIES + 1);
    bodies.push(Body::central());
    bodies.extend((0..NUM_BODIES).map(|_| Body::random(BASE_G)));
    
    let mut last_update = Instant::now();
    let mut time_multiplier = 1.0;  // Controls simulation speed
    let mut gravity_multiplier = 1.0;  // Controls gravity strength

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle controls
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) {
            time_multiplier *= 1.1;
        }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) {
            time_multiplier *= 0.9;
        }
        if window.is_key_down(Key::Key1) {
            gravity_multiplier *= 0.9;
        }
        if window.is_key_down(Key::Key2) {
            gravity_multiplier *= 1.1;
        }

        // Clear buffer with very dark blue background
        buffer.fill(0x000008);

        // Update physics with fixed timestep and substeps for smoothness
        let now = Instant::now();
        let elapsed = now.duration_since(last_update).as_secs_f32();
        
        let g = BASE_G * gravity_multiplier;
        let dt = BASE_DT;  // Keep base timestep constant
        
        // Calculate number of physics steps needed
        let steps_needed = ((elapsed * time_multiplier) / dt) as usize;
        let substeps = (steps_needed as f32 / 4.0).ceil() as usize;  // Limit max steps per frame
        
        if substeps > 0 {
            let adjusted_dt = (elapsed * time_multiplier) / substeps as f32;
            
            for _ in 0..substeps {
                let forces = calculate_forces(&bodies, g);
                for (body, force) in bodies.iter_mut().zip(forces) {
                    body.update(force, adjusted_dt);
                }
            }
        }
        
        last_update = now;

        // Draw bodies (central body last to overlay its glow)
        for body in &bodies[1..] {
            draw_circle(&mut buffer, body.pos, body.radius(), body.color, false);
        }
        // Draw central body with glow
        draw_circle(&mut buffer, bodies[0].pos, bodies[0].radius(), 0xFFAA33, true);

        // Update window title with controls and current multipliers
        window.set_title(&format!(
            "N-Body Simulation (4K) - Speed: {:.1}x (±) - Gravity: {:.1}x (1/2) - ESC to exit",
            time_multiplier, gravity_multiplier
        ));

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
