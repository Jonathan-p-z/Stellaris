use std::f32::consts::TAU;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use raylib::prelude::*;

const RAYON_DISQUE: f32 = 2.2;

pub struct BlackHole {
    pub pos: Vector3,
    pub rayon_influence: f32, // utilisé pour le lensing dans renderer
    pub disque: Vec<(Vector3, f32, Color)>, // (pos, taille, couleur)
}

impl BlackHole {
    pub fn new(pos: Vector3, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let nb = 36usize;
        let mut disque = Vec::with_capacity(nb);

        for i in 0..nb {
            let angle = (i as f32 / nb as f32) * TAU;
            // léger jitter autour du rayon pour pas avoir un anneau parfait/robotique
            let r = RAYON_DISQUE + rng.gen_range(-0.5..0.5f32);
            let bx = pos.x + r * angle.cos();
            let bz = pos.z + r * angle.sin();
            let by = pos.y + rng.gen_range(-0.1..0.1f32);

            // blanc chaud au cœur, orange-rouge à l'extérieur
            let heat = rng.gen::<f32>();
            let col = if heat > 0.65 {
                Color::new(255, 245, 190, 255)
            } else {
                Color::new(255, (70.0 + heat * 130.0) as u8, 15, 255)
            };

            let taille = rng.gen_range(0.22..0.55f32);
            disque.push((Vector3::new(bx, by, bz), taille, col));
        }

        BlackHole {
            pos,
            rayon_influence: RAYON_DISQUE * 2.5,
            disque,
        }
    }
}
