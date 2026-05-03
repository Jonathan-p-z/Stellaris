use raylib::prelude::*;

// Classification spectrale de Harvard — du plus chaud au plus froid
pub enum StarType {
    O, // bleu-violet, rare et massive
    B, // bleu
    A, // blanc-bleu
    F, // blanc chaud
    G, // jaune (notre soleil c'est un G)
    K, // orange
    M, // rouge, la majorité des étoiles de la galaxie
}

impl StarType {
    pub fn color(&self) -> Color {
        match self {
            StarType::O => Color::new(155, 176, 255, 255),
            StarType::B => Color::new(170, 191, 255, 255),
            StarType::A => Color::new(213, 224, 255, 255),
            StarType::F => Color::new(248, 247, 255, 255),
            StarType::G => Color::new(255, 244, 200, 255),
            StarType::K => Color::new(255, 210, 140, 255),
            StarType::M => Color::new(255, 150, 80, 255),
        }
    }

    #[allow(dead_code)]
    pub fn size(&self) -> f32 {
        match self {
            StarType::O => 14.0,
            StarType::B => 11.0,
            StarType::A => 9.0,
            StarType::F => 7.5,
            StarType::G => 6.0,
            StarType::K => 5.0,
            StarType::M => 3.5,
        }
    }

    // taille du billboard en unités scène
    pub fn billboard_size(&self) -> f32 {
        match self {
            StarType::O => 2.2,
            StarType::B => 1.8,
            StarType::A => 1.4,
            StarType::F => 1.1,
            StarType::G => 0.9,
            StarType::K => 0.7,
            StarType::M => 0.5,
        }
    }

    // distribution IMF approximative, la grande majorité des étoiles sont des naines M
    pub fn from_roll(roll: f32) -> Self {
        // cumulatif
        if roll < 0.0001 { StarType::O }
        else if roll < 0.0014 { StarType::B }
        else if roll < 0.0074 { StarType::A }
        else if roll < 0.0374 { StarType::F }
        else if roll < 0.113  { StarType::G }
        else if roll < 0.234  { StarType::K }
        else                  { StarType::M }
    }
}

pub struct Star {
    pub pos: Vector3,
    pub star_type: StarType,
    pub luminosite: f32, // 0.0 - 1.0
}
