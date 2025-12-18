// taken from simon0302010/efisnake

use uefi::boot;
use uefi::proto::rng::Rng as EfiRng;

pub struct Rng {
    pub state: f64,
}

impl Rng {
    /// creates a new rng
    pub fn new() -> Self {
        let seed = match boot::get_handle_for_protocol::<EfiRng>() {
            Ok(handle) => match boot::open_protocol_exclusive::<EfiRng>(handle) {
                Ok(mut rng) => {
                    let mut buf = [0u8; 8];
                    if rng.get_rng(None, &mut buf).is_ok() {
                        u64::from_le_bytes(buf) as f64
                    } else {
                        12345678901234567.0
                    }
                }
                Err(_) => 12345678901234567.0,
            },
            Err(_) => 12345678901234567.0,
        };

        Self { state: seed }
    }

    /// generates a random float in the specified range [min, max)
    pub fn random_range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.random_float()
    }

    /// generates a random float from 0.0 to 1.0
    pub fn random_float(&mut self) -> f64 {
        const A: u64 = 13891176665706064842;
        const C: u64 = 2227057010910366687;

        let next = (A.wrapping_mul(self.state as u64)).wrapping_add(C);
        self.state = next as f64;
        self.state / (u64::MAX as f64 + 1.0)
    }

    pub fn random_bool(&mut self, chance: f64) -> bool {
        self.random_float() < chance
    }
}
