use raylib::prelude::*;

use crate::blackhole::BlackHole;
use crate::events::{Event, EventType};
use crate::nebula::Nebula;
use crate::star::Star;

pub fn draw_stars(
    stars: &[Star],
    d: &mut impl RaylibDraw3D,
    tex: &Texture2D,
    cam: Camera3D,
    trous_noirs: &[BlackHole],
    centre_galaxie: Vector3,
) {
    let up = cam.up;
    let origin = Vector2::new(0.0, 0.0);
    let src = Rectangle { x: 0.0, y: 0.0, width: 64.0, height: 64.0 };

    for star in stars {
        let base = star.star_type.color();
        let lum = star.luminosite;
        let mut r = (base.r as f32 * lum) as u8;
        let mut g = (base.g as f32 * lum) as u8;
        let mut b = (base.b as f32 * lum) as u8;

        // lensing + grossissement noyau galactique
        let mut size_mult = 1.0f32;
        for bh in trous_noirs {
            let dx = star.pos.x - bh.pos.x;
            let dy = star.pos.y - bh.pos.y;
            let dz = star.pos.z - bh.pos.z;
            let dist = (dx*dx + dy*dy + dz*dz).sqrt();
            if dist < 3.0       { size_mult = 9.0; } // noyau dense — min 2.5u pour O/B
            else if dist < 5.0  { size_mult = 1.8; }
            let rayon_effet = bh.rayon_influence * 3.0;
            if dist < rayon_effet {
                let t = (1.0 - dist / rayon_effet) * 0.75;
                r = (r as f32 + (255.0 - r as f32) * t) as u8;
                g = (g as f32 + (220.0 - g as f32) * t * 0.6) as u8;
                b = (b as f32 + (255.0 - b as f32) * t) as u8;
            }
        }

        let col = Color::new(r, g, b, 255);

        let dx = star.pos.x - cam.position.x;
        let dy = star.pos.y - cam.position.y;
        let dz = star.pos.z - cam.position.z;
        let dist_sq_cam = dx*dx + dy*dy + dz*dz;

        if dist_sq_cam >= 200.0 * 200.0 {
            d.draw_point3D(star.pos, col);
        } else {
            let dist_cam = dist_sq_cam.sqrt();
            let s;
            if dist_cam < 15.0 {
                s = (star.star_type.billboard_size() * size_mult * 2.5).min(3.5);
                let col_sat = Color::new(
                    (r as f32 * 1.4).min(255.0) as u8,
                    (g as f32 * 1.4).min(255.0) as u8,
                    (b as f32 * 1.4).min(255.0) as u8,
                    255,
                );
                d.draw_billboard_pro(cam, *tex.as_ref(), src, star.pos, up, Vector2::new(s, s), origin, 0.0, col_sat);
            } else if dist_cam < 40.0 {
                s = (star.star_type.billboard_size() * size_mult * 1.5).min(2.5);
                d.draw_billboard_pro(cam, *tex.as_ref(), src, star.pos, up, Vector2::new(s, s), origin, 0.0, col);
            } else {
                let fade = if dist_cam > 160.0 { 1.0 - (dist_cam - 160.0) / 40.0 } else { 1.0 };
                s = star.star_type.billboard_size() * size_mult * fade.max(0.05);
                d.draw_billboard_pro(cam, *tex.as_ref(), src, star.pos, up, Vector2::new(s, s), origin, 0.0, col);
            }

            // pas de bloom dans le noyau dense
            let dx_c = star.pos.x - centre_galaxie.x;
            let dy_c = star.pos.y - centre_galaxie.y;
            let dz_c = star.pos.z - centre_galaxie.z;
            let dist_centre = (dx_c*dx_c + dy_c*dy_c + dz_c*dz_c).sqrt();
            let faire_bloom = dist_centre > 8.0;

            if faire_bloom {
                // halo bloom 1
                let col_halo1 = Color::new(
                    (col.r as f32 * 0.35) as u8,
                    (col.g as f32 * 0.35) as u8,
                    (col.b as f32 * 0.35) as u8,
                    255,
                );
                d.draw_billboard_pro(cam, *tex.as_ref(), src, star.pos, cam.up, Vector2::new(s * 2.5, s * 2.5), origin, 0.0, col_halo1);

                // halo bloom 2
                let col_halo2 = Color::new(
                    (col.r as f32 * 0.12) as u8,
                    (col.g as f32 * 0.12) as u8,
                    (col.b as f32 * 0.12) as u8,
                    255,
                );
                d.draw_billboard_pro(cam, *tex.as_ref(), src, star.pos, cam.up, Vector2::new(s * 5.0, s * 5.0), origin, 0.0, col_halo2);
            }
        }
    }
}

// nébuleuse en blend alpha, draw_billboard simple, toujours face caméra
pub fn draw_nebula(neb: &Nebula, d: &mut impl RaylibDraw3D, tex: &Texture2D, cam: Camera3D) {
    for (pos, taille, _col) in &neb.nuages {
        d.draw_billboard(cam, tex, *pos, *taille * 0.4, neb.couleur);
    }
}

pub fn draw_events(events: &[Event], d: &mut impl RaylibDraw3D, tex: &Texture2D, cam: Camera3D, temps: f32) {
    let up = cam.up;
    let origin = Vector2::new(0.0, 0.0);
    let src = Rectangle { x: 0.0, y: 0.0, width: 64.0, height: 64.0 };

    for (index, ev) in events.iter().enumerate() {
        match &ev.kind {
            EventType::Supernova => {
                let taille = 4.75 + (temps * 2.0 + index as f32).sin() * 1.25;
                d.draw_billboard_pro(cam, *tex.as_ref(), src, ev.pos, up,
                    Vector2::new(taille * 3.0, taille * 3.0), origin, 0.0, Color::new(51, 49, 40, 255));
                d.draw_billboard_pro(cam, *tex.as_ref(), src, ev.pos, up,
                    Vector2::new(taille * 1.8, taille * 1.8), origin, 0.0, Color::new(255, 245, 200, 40));
                d.draw_billboard_pro(cam, *tex.as_ref(), src, ev.pos, up,
                    Vector2::new(taille, taille), origin, 0.0, Color::new(255, 245, 200, 255));
            }
            EventType::Pulsar => {
                let a = ((temps * 12.0 + index as f32 * 1.7).sin() + 1.0) * 0.5 * 200.0 + 55.0;
                d.draw_billboard_pro(cam, *tex.as_ref(), src, ev.pos, up,
                    Vector2::new(5.0, 5.0), origin, 0.0, Color::new(180, 220, 255, 40));
                d.draw_billboard_pro(cam, *tex.as_ref(), src, ev.pos, up,
                    Vector2::new(2.5, 2.5), origin, 0.0, Color::new(180, 220, 255, a as u8));
            }
            EventType::AmasGlobulaire => {
                for pos in &ev.membres {
                    d.draw_point3D(*pos, Color::new(255, 240, 200, 255));
                }
            }
            EventType::EtoileBinaire => {
                for pos in &ev.membres {
                    d.draw_billboard_pro(cam, *tex.as_ref(), src, *pos, up,
                        Vector2::new(0.15, 0.15), origin, 0.0, Color::new(200, 220, 255, 255));
                }
            }
            EventType::Quasar => {
                let halo_ext = 22.0 + (temps * 0.8).sin() * 2.0;
                d.draw_billboard_pro(cam, *tex.as_ref(), src, ev.pos, up,
                    Vector2::new(halo_ext, halo_ext), origin, 0.0, Color::new(200, 100, 30, 35));
                d.draw_billboard_pro(cam, *tex.as_ref(), src, ev.pos, up,
                    Vector2::new(14.0, 14.0), origin, 0.0, Color::new(255, 150, 50, 80));
                d.draw_billboard_pro(cam, *tex.as_ref(), src, ev.pos, up,
                    Vector2::new(7.0, 7.0), origin, 0.0, Color::new(255, 200, 100, 255));
            }
        }
    }
}

// disque d'accrétion en additif, billboards chauds autour du centre
pub fn draw_blackhole_disque(bh: &BlackHole, d: &mut impl RaylibDraw3D, tex: &Texture2D, cam: Camera3D) {
    let up = cam.up;
    let origin = Vector2::new(0.0, 0.0);
    let src = Rectangle { x: 0.0, y: 0.0, width: 64.0, height: 64.0 };
    for (pos, taille, col) in &bh.disque {
        d.draw_billboard_pro(cam, *tex.as_ref(), src, *pos, up, Vector2::new(*taille, *taille), origin, 0.0, *col);
    }
}
