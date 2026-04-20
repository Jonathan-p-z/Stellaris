use std::f32::consts::TAU;

use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use raylib::prelude::*;

use crate::star::{Star, StarType};

pub struct Galaxy {
    pub centre: Vector3,
    pub rayon: f32,
    pub nb_bras: u32,
    pub inclinaison: f32,
    pub etoiles: Vec<Star>,
}

impl Galaxy {
    // biais: -1.0 = jeune (bleue, beaucoup d'O/B), +1.0 = vieille (rouge, K/M dominants)
    // inclinaison: angle en radians, rotation sur X — pour que les 3 galaxies soient pas toutes plates
    pub fn generate(centre: Vector3, rayon: f32, nb_etoiles: u32, seed: u64, inclinaison: f32, biais: f32) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let perlin = Perlin::new(seed as u32);
        let nb_bras = rng.gen_range(3u32..=5);

        let mut etoiles = Vec::with_capacity(nb_etoiles as usize);

        // bulge central
        let nb_bulge = (nb_etoiles as f32 * 0.15) as u32;
        for _ in 0..nb_bulge {
            let r = rng.gen::<f32>().powf(2.0) * rayon * 0.18;
            let theta = rng.gen::<f32>() * TAU;
            let phi = rng.gen::<f32>() * TAU;
            let x = centre.x + r * theta.cos();
            let z = centre.z + r * theta.sin();
            let y = centre.y + r * 0.3 * phi.sin();

            // noyau toujours chaud écrase le biais galaxie, ~40% O/B
            let roll_b = rng.gen::<f32>() * 0.004;
            let star_type = StarType::from_roll(roll_b);
            let luminosite = rng.gen_range(0.5f32..1.0);
            let taille = star_type.size() * rng.gen_range(0.8f32..1.5);

            etoiles.push(Star { pos: Vector3::new(x, y, z), star_type, taille, luminosite });
        }

        // bras spiraux
        let nb_bras_etoiles = nb_etoiles - nb_bulge;
        let spiral_turns = 2.5f32;

        for _ in 0..nb_bras_etoiles {
            let bras = rng.gen_range(0..nb_bras);
            let offset_bras = (bras as f32 / nb_bras as f32) * TAU;

            let t = rng.gen::<f32>().powf(1.8);
            let r = t * rayon;
            let theta = offset_bras + t * spiral_turns * TAU;

            let ns = 0.006f64;
            let bruit_x = perlin.get([r as f64 * ns, theta as f64 * 0.4]) as f32;
            let bruit_z = perlin.get([r as f64 * ns + 73.1, theta as f64 * 0.4 + 31.7]) as f32;
            let spread = rayon * 0.065 * (0.25 + t);

            let x = centre.x + r * theta.cos() + bruit_x * spread;
            let z = centre.z + r * theta.sin() + bruit_z * spread;
            let y_range = rayon * 0.025 * (1.0 - t * 0.8).max(0.015);
            let y = centre.y + rng.gen_range(-y_range..y_range);

            let roll_b = biaiser(rng.gen::<f32>(), biais);
            let star_type = StarType::from_roll(roll_b);
            let luminosite = rng.gen_range(0.35f32..1.0);
            let taille = star_type.size() * rng.gen_range(0.7f32..1.4);

            etoiles.push(Star { pos: Vector3::new(x, y, z), star_type, taille, luminosite });
        }

        // appliquer l'inclinaison, rotation sur X autour du centre de la galaxie
        if inclinaison.abs() > 0.001 {
            let (sin_i, cos_i) = inclinaison.sin_cos();
            for star in &mut etoiles {
                let ry = star.pos.y - centre.y;
                let rz = star.pos.z - centre.z;
                star.pos.y = centre.y + ry * cos_i - rz * sin_i;
                star.pos.z = centre.z + ry * sin_i + rz * cos_i;
            }
        }

        Galaxy { centre, rayon, nb_bras, inclinaison, etoiles }
    }
}

// biais sur le roll de sélection spectrale
// biais < 0 = jeune/chaud (compresse vers 0 = O/B)
// biais > 0 = vieux/froid (étire vers 1 = K/M)
fn biaiser(roll: f32, biais: f32) -> f32 {
    if biais < 0.0 {
        (roll * (1.0 + biais * 0.8)).clamp(0.001, 0.999)
    } else {
        (roll + (1.0 - roll) * biais * 0.8).clamp(0.001, 0.999)
    }
}
