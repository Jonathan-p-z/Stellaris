use std::f32::consts::TAU;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use raylib::prelude::*;

pub enum EventType {
    Supernova,
    Pulsar,
    AmasGlobulaire,
    EtoileBinaire,
    Quasar,
}

pub struct Event {
    pub pos: Vector3,
    pub kind: EventType,
    pub membres: Vec<Vector3>,
}

pub fn generer_events(seed: u64, centres_galaxies: &[Vector3]) -> Vec<Event> {
    let mut rng = StdRng::seed_from_u64(seed);

    // (type_idx, nb) ordre fixe pour seed reproductible
    let quota = [(0u8, 12u32), (1, 10), (2, 8), (3, 8), (4, 2)];

    let mut events = Vec::new();
    for (type_idx, nb) in quota {
        for _ in 0..nb {
            let pos = gen_pos(&mut rng, centres_galaxies);
            let (kind, membres) = match type_idx {
                0 => (EventType::Supernova,     vec![]),
                1 => (EventType::Pulsar,         vec![]),
                2 => (EventType::AmasGlobulaire, gen_amas(&mut rng, pos)),
                3 => (EventType::EtoileBinaire,  gen_binaire(&mut rng, pos)),
                _ => (EventType::Quasar,         vec![]),
            };
            events.push(Event { pos, kind, membres });
        }
    }
    events
}

fn gen_pos(rng: &mut StdRng, centres: &[Vector3]) -> Vector3 {
    loop {
        let theta = rng.gen::<f32>() * TAU;
        let phi = (rng.gen::<f32>() * 2.0 - 1.0).acos();
        let r = rng.gen_range(50.0..400.0f32);
        let pos = Vector3::new(
            r * phi.sin() * theta.cos(),
            r * phi.cos(),
            r * phi.sin() * theta.sin(),
        );
        if centres.iter().all(|c| dist3(pos, *c) >= 30.0) {
            return pos;
        }
    }
}

fn gen_amas(rng: &mut StdRng, centre: Vector3) -> Vec<Vector3> {
    let nb = rng.gen_range(40..=80usize);
    (0..nb).map(|_| {
        let theta = rng.gen::<f32>() * TAU;
        let phi = (rng.gen::<f32>() * 2.0 - 1.0).acos();
        let r = rng.gen::<f32>() * 2.0;
        Vector3::new(
            centre.x + r * phi.sin() * theta.cos(),
            centre.y + r * phi.cos(),
            centre.z + r * phi.sin() * theta.sin(),
        )
    }).collect()
}

fn gen_binaire(rng: &mut StdRng, centre: Vector3) -> Vec<Vector3> {
    let offset = rng.gen_range(0.3..0.8f32);
    let theta = rng.gen::<f32>() * TAU;
    vec![
        Vector3::new(centre.x + offset * theta.cos(), centre.y, centre.z + offset * theta.sin()),
        Vector3::new(centre.x - offset * theta.cos(), centre.y, centre.z - offset * theta.sin()),
    ]
}

fn dist3(a: Vector3, b: Vector3) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
}
