mod blackhole;
mod camera;
mod events;
mod galaxy;
mod nebula;
mod renderer;
mod star;

use std::f32::consts::TAU;

use image::{ImageBuffer, Rgba};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use raylib::prelude::*;

use blackhole::BlackHole;
use camera::Cam;
use galaxy::Galaxy;
use nebula::Nebula;
use star::{Star, StarType};

struct Config {
    nb_etoiles_par_galaxie:  usize,
    nb_poussiere_sphere:     usize,
    nb_poussiere_disque:     usize,
    seuil_culling_etoiles:   f32,
    seuil_billboard:         f32,
    seuil_culling_poussiere: f32,
    bloom_actif:             bool,
}

impl Config {
    fn high() -> Self {
        Self {
            nb_etoiles_par_galaxie:  25_000,
            nb_poussiere_sphere:     40_000,
            nb_poussiere_disque:     15_000,
            seuil_culling_etoiles:   450.0,
            seuil_billboard:          60.0,
            seuil_culling_poussiere: 300.0,
            bloom_actif:             true,
        }
    }
    fn low() -> Self {
        Self {
            nb_etoiles_par_galaxie:  13_000,
            nb_poussiere_sphere:     20_000,
            nb_poussiere_disque:      8_000,
            seuil_culling_etoiles:   300.0,
            seuil_billboard:          40.0,
            seuil_culling_poussiere: 200.0,
            bloom_actif:             false,
        }
    }
}

fn point_in_rect(p: Vector2, rx: i32, ry: i32, rw: i32, rh: i32) -> bool {
    p.x >= rx as f32 && p.x < (rx + rw) as f32
        && p.y >= ry as f32 && p.y < (ry + rh) as f32
}

struct EtoileFilante {
    pos_depart: Vector3,
    pos_arrivee: Vector3,
    t_debut: f32,
    duree: f32,
}

fn generer_textures_si_besoin() {
    std::fs::create_dir_all("assets").ok();

    if !std::path::Path::new("assets/star_halo.png").exists() {
        let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(64, 64);
        for y in 0u32..64 {
            for x in 0u32..64 {
                let d = (((x as f32 - 32.0).powi(2) + (y as f32 - 32.0).powi(2)).sqrt()) / 32.0;
                // gaussienne pure — pas de bord dur, dégradé continu vers 0
                let a = ((-d * d / 0.18).exp() * 255.0) as u8;
                img.put_pixel(x, y, Rgba([255, 255, 255, a]));
            }
        }
        img.save("assets/star_halo.png").expect("impossible d'écrire star_halo.png");
    }

    if !std::path::Path::new("assets/nebula_cloud.png").exists() {
        let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(256, 256);
        for y in 0u32..256 {
            for x in 0u32..256 {
                let d = (((x as f32 - 128.0).powi(2) + (y as f32 - 128.0).powi(2)).sqrt()) / 128.0;
                if d < 1.0 {
                    let base = (1.0 - d).powi(3) * 85.0;
                    // bruit pour casser la symétrie parfaite
                    let bruit = 0.7 + 0.3 * (x as f32 * 0.4).sin() * (y as f32 * 0.37).cos();
                    let a = (base * bruit).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([255, 255, 255, a]));
                }
            }
        }
        img.save("assets/nebula_cloud.png").expect("impossible d'écrire nebula_cloud.png");
    }
}

fn generer_poussiere(nb_sphere: usize, nb_disque: usize) -> Vec<(Vector3, Color, bool)> {
    let mut rng = StdRng::seed_from_u64(999);
    let mut dust = Vec::with_capacity(nb_sphere + nb_disque);

    // couche fond galactique, sphère 360° rayon 80-600u
    for _ in 0..nb_sphere {
        let theta = rng.gen::<f32>() * TAU;
        let phi = (rng.gen::<f32>() * 2.0 - 1.0).acos();
        let r = rng.gen_range(80.0..600.0f32);
        let x = r * phi.sin() * theta.cos();
        let y = r * phi.cos();
        let z = r * phi.sin() * theta.sin();
        let v = rng.gen_range(55u8..110);
        let a = rng.gen_range(100u8..230);
        let teinte = rng.gen_range(0u8..100);
        let col = if teinte < 15 {
            Color::new(v, v, (v as f32 * 1.3).min(255.0) as u8, a)
        } else if teinte < 25 {
            Color::new((v as f32 * 1.2).min(255.0) as u8, v, (v as f32 * 0.7) as u8, a)
        } else {
            Color::new(v, v, v, a)
        };
        let is_bright = rng.gen::<f32>() < 0.08;
        dust.push((Vector3::new(x, y, z), col, is_bright));
    }

    // couche voie lactée locale disque aplati rayon 500u, y gaussien σ≈8
    for _ in 0..nb_disque {
        let theta = rng.gen::<f32>() * TAU;
        let r = rng.gen::<f32>().sqrt() * 500.0;
        let x = r * theta.cos();
        let z = r * theta.sin();
        // somme de 3 uniforms ≈ gaussienne, σ ~ 8u
        let gy = (rng.gen::<f32>() + rng.gen::<f32>() + rng.gen::<f32>()) / 3.0 - 0.5;
        let y = gy * 48.0;
        let v = rng.gen_range(60u8..100);
        let a = rng.gen_range(100u8..220);
        // blanc chaud légèrement jaunâtre
        dust.push((Vector3::new(x, y, z), Color::new(v, (v as f32 * 0.9) as u8, (v as f32 * 0.7) as u8, a), false));
    }

    dust
}

fn main() {
    generer_textures_si_besoin();

    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("Stellaris")
        .vsync()
        .build();

    // — menu de sélection de qualité —
    let config;
    {
        // mesure les textes une seule fois avant la boucle
        let tw_titre = rl.measure_text("STELLARIS", 60);
        let tw_sub   = rl.measure_text("Choisissez votre mode de rendu", 20);
        let tw_bh    = rl.measure_text("HIGH - GPU dédié recommandé", 22);
        let tw_bl    = rl.measure_text("LOW - Compatible iGPU / moyenne gamme", 22);
        let tw_dh    = rl.measure_text("75 600 étoiles · Bloom post-process · Rendu complet", 14);
        let tw_dl    = rl.measure_text("39 000 étoiles · Sans bloom · Optimisé CPU", 14);

        loop {
            if rl.window_should_close() { return; }

            let mp   = rl.get_mouse_position();
            let clic = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
            let hover_high = point_in_rect(mp, 390, 270, 500, 75);
            let hover_low  = point_in_rect(mp, 390, 400, 500, 75);

            {
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::BLACK);

                // titre
                d.draw_text("STELLARIS", 640 - tw_titre / 2, 100, 60, Color::WHITE);
                d.draw_text("Choisissez votre mode de rendu", 640 - tw_sub / 2, 185, 20, Color::new(170, 170, 195, 255));

                // — bouton HIGH —
                let col_high = if hover_high { Color::new(230, 175, 65, 255) } else { Color::new(195, 140, 35, 255) };
                d.draw_rectangle(390, 270, 500, 75, col_high);
                d.draw_text("HIGH - GPU dédié recommandé", 640 - tw_bh / 2, 270 + (75 - 22) / 2, 22, Color::WHITE);
                d.draw_text("75 600 étoiles · Bloom post-process · Rendu complet", 640 - tw_dh / 2, 358, 14, Color::new(180, 180, 195, 200));

                // — bouton LOW —
                let col_low = if hover_low { Color::new(110, 145, 180, 255) } else { Color::new(80, 110, 145, 255) };
                d.draw_rectangle(390, 400, 500, 75, col_low);
                d.draw_text("LOW - Compatible iGPU / moyenne gamme", 640 - tw_bl / 2, 400 + (75 - 22) / 2, 22, Color::WHITE);
                d.draw_text("39 000 étoiles · Sans bloom · Optimisé CPU", 640 - tw_dl / 2, 488, 14, Color::new(180, 180, 195, 200));
            }

            if clic && hover_high { config = Config::high(); break; }
            if clic && hover_low  { config = Config::low();  break; }
        }
    }

    rl.disable_cursor();

    // 3 galaxies avec inclinaison et biais spectral différents
    let nb_eg = config.nb_etoiles_par_galaxie as u32;
    let mut galaxies = vec![
        Galaxy::generate(Vector3::new(0.0, 0.0, 0.0),       100.0, nb_eg, 42,  -0.22, -0.75), // jeune, O/B dominant
        Galaxy::generate(Vector3::new(280.0, 12.0, -55.0),   85.0, nb_eg, 137,  0.18,  0.65), // vieille, G/K dominant
        Galaxy::generate(Vector3::new(-210.0, -8.0, 165.0),  75.0, nb_eg, 271,  0.27,  0.35), // irrégulière, légère dominante K/M
    ];

    // renforce les noyaux 200 O/B supplémentaires dans rayon 3u par galaxie
    {
        let mut rng_noyau = StdRng::seed_from_u64(9999);
        for g in &mut galaxies {
            for _ in 0..200 {
                let theta = rng_noyau.gen::<f32>() * TAU;
                let phi = (rng_noyau.gen::<f32>() * 2.0 - 1.0).acos();
                let r = rng_noyau.gen::<f32>() * 3.0;
                let pos = Vector3::new(
                    g.centre.x + r * phi.sin() * theta.cos(),
                    g.centre.y + r * phi.cos(),
                    g.centre.z + r * phi.sin() * theta.sin(),
                );
                let st = if rng_noyau.gen::<f32>() < 0.15 { StarType::O } else { StarType::B };
                let luminosite = rng_noyau.gen_range(0.8..1.0f32);
                g.etoiles.push(Star { pos, star_type: st, luminosite });
            }
        }
    }

    // 1 trou noir par galaxie au centre
    let trous_noirs = vec![
        BlackHole::new(Vector3::new(0.0, 0.0, 0.0),       1001),
        BlackHole::new(Vector3::new(280.0, 12.0, -55.0),  1002),
        BlackHole::new(Vector3::new(-210.0, -8.0, 165.0), 1003),
    ];

    // 4 nébuleuses par galaxie, positionnées dans les bras
    let couleurs_neb = [
        Color::new(153, 178, 255, 255), // G0 bleutée
        Color::new(255, 178,  76, 255), // G1 orangée
        Color::new(255, 102, 102, 255), // G2 rougeâtre
    ];
    let mut nebuleuses = Vec::new();
    {
        let mut rng_neb = StdRng::seed_from_u64(700);
        for g_idx in 0..3usize {
            let c = galaxies[g_idx].centre;
            for i in 0..4usize {
                let theta = rng_neb.gen::<f32>() * TAU;
                let r = rng_neb.gen_range(10.0..80.0f32);
                let nx = c.x + r * theta.cos();
                let ny = c.y + rng_neb.gen_range(-10.0..10.0f32);
                let nz = c.z + r * theta.sin();
                nebuleuses.push(Nebula::new(
                    Vector3::new(nx, ny, nz),
                    couleurs_neb[g_idx],
                    600 + g_idx as u64 * 10 + i as u64,
                ));
            }
        }
    }

    let centres = [galaxies[0].centre, galaxies[1].centre, galaxies[2].centre];
    let events_liste = events::generer_events(800, &centres);

    // pool de 8 étoiles filantes — toutes inactives au démarrage
    let mut filantes: Vec<EtoileFilante> = (0..8).map(|_| EtoileFilante {
        pos_depart: Vector3::zero(),
        pos_arrivee: Vector3::zero(),
        t_debut: -999.0,
        duree: 2.0,
    }).collect();
    let mut rng_filantes = StdRng::seed_from_u64(12345);

    let nb_total: usize = galaxies.iter().map(|g| g.etoiles.len()).sum();
    let poussiere = generer_poussiere(config.nb_poussiere_sphere, config.nb_poussiere_disque);

    let mut tex_glow   = rl.load_texture(&thread, "assets/star_halo.png").unwrap();
    let mut tex_nebula = rl.load_texture(&thread, "assets/nebula_cloud.png").unwrap();
    // mipmaps d'abord, sinon le trilinéaire n'a rien à interpoler
    tex_glow.gen_texture_mipmaps();
    tex_nebula.gen_texture_mipmaps();
    tex_glow.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_TRILINEAR);
    tex_nebula.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_TRILINEAR);

    let mut cam = Cam::new();

    // render texture pour capturer la scène avant le post-process
    let mut render_tex = rl.load_render_texture(&thread, 1280, 720).unwrap();
    let mut bloom_shader = rl.load_shader(&thread, None, Some("assets/shaders/bloom.glsl"));
    let res_loc = bloom_shader.get_shader_location("resolution");
    bloom_shader.set_shader_value(res_loc, Vector2::new(1280.0, 720.0));

    while !rl.window_should_close() {
        cam.update(&rl);
        let temps = rl.get_time() as f32;

        // — pass 1 : scène entière dans la render texture —
        {
            let mut d = rl.begin_texture_mode(&thread, &mut *render_tex);
            d.clear_background(Color::BLACK);

            {
                let mut d3 = d.begin_mode3D(cam.inner);

                unsafe { raylib::ffi::rlDisableDepthMask(); }

                {
                    let mut da = d3.begin_blend_mode(BlendMode::BLEND_ALPHA);
                    for neb in &nebuleuses {
                        renderer::draw_nebula(neb, &mut da, &tex_nebula, cam.inner);
                    }
                    for bh in &trous_noirs {
                        da.draw_sphere(bh.pos, 0.45, Color::BLACK);
                    }
                }

                {
                    let mut db = d3.begin_blend_mode(BlendMode::BLEND_ADDITIVE);

                    let cam_pos = cam.inner.position;
                    let scp = config.seuil_culling_poussiere * config.seuil_culling_poussiere;
                    for (pos, col, is_bright) in &poussiere {
                        let dx = pos.x - cam_pos.x;
                        let dy = pos.y - cam_pos.y;
                        let dz = pos.z - cam_pos.z;
                        if dx*dx + dy*dy + dz*dz > scp { continue; }

                        if *is_bright {
                            db.draw_billboard(cam.inner, &tex_glow, *pos, 0.6, *col);
                        } else {
                            db.draw_point3D(*pos, *col);
                        }
                    }

                    for g in &galaxies {
                        renderer::draw_stars(
                            &g.etoiles, &mut db, &tex_glow, cam.inner, &trous_noirs, g.centre,
                            config.seuil_culling_etoiles, config.seuil_billboard,
                        );
                    }

                    for bh in &trous_noirs {
                        renderer::draw_blackhole_disque(bh, &mut db, &tex_glow, cam.inner);
                    }

                    renderer::draw_events(&events_liste, &mut db, &tex_glow, cam.inner, temps);

                    for f in &mut filantes {
                        let active = f.t_debut >= 0.0 && (temps - f.t_debut) < f.duree;
                        if !active {
                            if rng_filantes.gen::<f32>() < 0.003 {
                                let theta = rng_filantes.gen::<f32>() * TAU;
                                let phi = (rng_filantes.gen::<f32>() * 2.0 - 1.0).acos();
                                let r = rng_filantes.gen_range(50.0..300.0f32);
                                f.pos_depart = Vector3::new(
                                    r * phi.sin() * theta.cos(),
                                    r * phi.cos(),
                                    r * phi.sin() * theta.sin(),
                                );
                                let longueur = rng_filantes.gen_range(15.0..40.0f32);
                                let dtheta = rng_filantes.gen::<f32>() * TAU;
                                f.pos_arrivee = Vector3::new(
                                    f.pos_depart.x + longueur * dtheta.cos(),
                                    f.pos_depart.y + rng_filantes.gen_range(-5.0..5.0f32),
                                    f.pos_depart.z + longueur * dtheta.sin(),
                                );
                                f.duree = rng_filantes.gen_range(1.5..3.0f32);
                                f.t_debut = temps;
                            }
                        } else {
                            let t = (temps - f.t_debut) / f.duree;
                            let pos = Vector3::new(
                                f.pos_depart.x + (f.pos_arrivee.x - f.pos_depart.x) * t,
                                f.pos_depart.y + (f.pos_arrivee.y - f.pos_depart.y) * t,
                                f.pos_depart.z + (f.pos_arrivee.z - f.pos_depart.z) * t,
                            );
                            db.draw_point3D(pos, Color::new(255, 255, 255, 255));
                            for i in 1..=3usize {
                                let t_trail = (t - i as f32 * 0.04).max(0.0);
                                let p = Vector3::new(
                                    f.pos_depart.x + (f.pos_arrivee.x - f.pos_depart.x) * t_trail,
                                    f.pos_depart.y + (f.pos_arrivee.y - f.pos_depart.y) * t_trail,
                                    f.pos_depart.z + (f.pos_arrivee.z - f.pos_depart.z) * t_trail,
                                );
                                let a = (200 - i as u8 * 60).max(40);
                                db.draw_point3D(p, Color::new(255, 255, 255, a));
                            }
                        }
                    }
                }

                unsafe { raylib::ffi::rlEnableDepthMask(); }
            }
        }

        // — pass 2 : bloom post-process + HUD —
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            // blit de la render texture — avec bloom shader ou sans selon la config
            if config.bloom_actif {
                let mut sd = d.begin_shader_mode(&mut bloom_shader);
                sd.draw_texture_pro(
                    &render_tex,
                    Rectangle { x: 0.0, y: 0.0, width: 1280.0, height: -720.0 },
                    Rectangle { x: 0.0, y: 0.0, width: 1280.0, height:  720.0 },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            } else {
                // la render texture raylib est flippée verticalement, height négatif corrige ça
                d.draw_texture_pro(
                    &render_tex,
                    Rectangle { x: 0.0, y: 0.0, width: 1280.0, height: -720.0 },
                    Rectangle { x: 0.0, y: 0.0, width: 1280.0, height:  720.0 },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }

            // — HUD —
            let (cx, cy) = (640i32, 360i32);
            let col_cross = Color::new(255, 255, 255, 100);
            d.draw_line(cx - 12, cy, cx - 5, cy, col_cross);
            d.draw_line(cx + 5,  cy, cx + 12, cy, col_cross);
            d.draw_line(cx, cy - 12, cx, cy - 5,  col_cross);
            d.draw_line(cx, cy + 5,  cx, cy + 12, col_cross);

            d.draw_fps(12, 12);
            d.draw_text(&format!("{} etoiles", nb_total), 12, 34, 16, Color::new(160, 160, 180, 170));

            let dots = ["*  .  .", "*  *  .", "*  *  *"];
            d.draw_text(dots[cam.palier() - 1], 12, 698, 16, Color::new(180, 180, 210, 160));
        }
    }
}
