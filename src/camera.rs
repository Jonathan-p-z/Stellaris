use raylib::prelude::*;

// 3 paliers : exploration lente, traversée normale, saut rapide
const VITESSES: [f32; 3] = [20.0, 40.0, 120.0];
const SENSIBILITE: f32 = 0.003;
const PITCH_MAX: f32 = 1.48;

pub struct Cam {
    pub inner: Camera3D,
    yaw: f32,
    pitch: f32,
    vitesse_idx: usize,
}

fn norm3(v: Vector3) -> Vector3 {
    let len = (v.x*v.x + v.y*v.y + v.z*v.z).sqrt();
    if len < 0.0001 { return Vector3::new(0.0, 0.0, 1.0); }
    Vector3::new(v.x / len, v.y / len, v.z / len)
}

impl Cam {
    pub fn new() -> Self {
        // en bordure de G0, on démarre dedans, pas en observateur externe
        let pos    = Vector3::new(0.0, 45.0, 120.0);
        let target = Vector3::new(0.0,  0.0,   0.0);

        let dir = norm3(Vector3::new(
            target.x - pos.x,
            target.y - pos.y,
            target.z - pos.z,
        ));
        // yaw/pitch extraits de la direction initiale
        let yaw   = dir.x.atan2(dir.z);
        let pitch = dir.y.asin().clamp(-PITCH_MAX, PITCH_MAX);

        let inner = Camera3D::perspective(pos, target, Vector3::new(0.0, 1.0, 0.0), 60.0);
        Cam { inner, yaw, pitch, vitesse_idx: 1 }
    }

    pub fn palier(&self) -> usize { self.vitesse_idx + 1 }

    pub fn update(&mut self, rl: &RaylibHandle) {
        let dt    = rl.get_frame_time();
        let speed = VITESSES[self.vitesse_idx];

        // molette = change de palier
        let scroll = rl.get_mouse_wheel_move();
        if scroll > 0.5 {
            self.vitesse_idx = (self.vitesse_idx + 1).min(VITESSES.len() - 1);
        } else if scroll < -0.5 {
            self.vitesse_idx = self.vitesse_idx.saturating_sub(1);
        }

        // orientation souris curseur capturé (disable_cursor), delta direct
        let delta = rl.get_mouse_delta();
        self.yaw   -= delta.x * SENSIBILITE;
        self.pitch  -= delta.y * SENSIBILITE;
        self.pitch   = self.pitch.clamp(-PITCH_MAX, PITCH_MAX);

        // vecteur forward depuis yaw/pitch stockés
        let forward = Vector3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        );

        // right = forward × (0,1,0), normalisé sur XZ
        let right = norm3(Vector3::new(-forward.z, 0.0, forward.x));

        // accumule le déplacement demandé
        let mut mov = Vector3::new(0.0, 0.0, 0.0);
        if rl.is_key_down(KeyboardKey::KEY_W) { mov.x += forward.x; mov.y += forward.y; mov.z += forward.z; }
        if rl.is_key_down(KeyboardKey::KEY_S) { mov.x -= forward.x; mov.y -= forward.y; mov.z -= forward.z; }
        if rl.is_key_down(KeyboardKey::KEY_D) { mov.x += right.x;   mov.z += right.z; }
        if rl.is_key_down(KeyboardKey::KEY_A) { mov.x -= right.x;   mov.z -= right.z; }
        if rl.is_key_down(KeyboardKey::KEY_SPACE) { mov.y += 1.0; }
        if rl.is_key_down(KeyboardKey::KEY_C)    { mov.y -= 1.0; }

        // normaliser la diagonale, sinon on va plus vite à 45°
        let mov_len = (mov.x*mov.x + mov.y*mov.y + mov.z*mov.z).sqrt();
        if mov_len > 0.001 {
            let d = speed * dt / mov_len;
            self.inner.position.x += mov.x * d;
            self.inner.position.y += mov.y * d;
            self.inner.position.z += mov.z * d;
        }

        // target suit la position, garde la direction, pas juste un point fixe
        self.inner.target.x = self.inner.position.x + forward.x;
        self.inner.target.y = self.inner.position.y + forward.y;
        self.inner.target.z = self.inner.position.z + forward.z;
    }
}
