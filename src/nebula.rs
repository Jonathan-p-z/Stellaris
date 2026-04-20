use std::f32::consts::TAU;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use raylib::prelude::*;

pub struct Nebula {
    pub couleur: Color,
    pub seed: u64,
    pub nuages: Vec<(Vector3, f32, Color)>,
}

impl Nebula {
    pub fn new(pos: Vector3, couleur: Color, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let nb = 60usize;
        let mut nuages = Vec::with_capacity(nb);
        let spread = 15.0f32;

        for _ in 0..nb {
            let r = rng.gen::<f32>().sqrt() * spread;
            let theta = rng.gen::<f32>() * TAU;
            let dy = (rng.gen::<f32>() - 0.5) * spread * 0.4;

            let bx = pos.x + r * theta.cos();
            let by = pos.y + dy;
            let bz = pos.z + r * theta.sin();
            let taille = rng.gen_range(8.0f32..16.0);

            // variation +/- 20 sur RGB pour casser l'aplat uniforme
            let dr = rng.gen_range(-20i32..=20);
            let dg = rng.gen_range(-20i32..=20);
            let db = rng.gen_range(-20i32..=20);
            let col = Color::new(
                (couleur.r as i32 + dr).clamp(0, 255) as u8,
                (couleur.g as i32 + dg).clamp(0, 255) as u8,
                (couleur.b as i32 + db).clamp(0, 255) as u8,
                80,
            );

            nuages.push((Vector3::new(bx, by, bz), taille, col));
        }

        Nebula { couleur, seed, nuages }
    }
}
