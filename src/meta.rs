//! Metadata structures for TCM format versions.

pub trait Meta: Send + Sync {
    fn size() -> usize where Self: Sized;
    fn tps(&self) -> f32;
    fn tps_dt(&self) -> f32;
    fn uses_dt(&self) -> bool;
    fn version() -> u8 where Self: Sized;
    fn version_instance(&self) -> u8;
    fn rng_seed(&self) -> Option<u64>;
    fn is_rng_seed_set(&self) -> bool;
    fn from_bytes(bytes: &[u8]) -> Self where Self: Sized;
    fn to_bytes(&self) -> Box<[u8]>;
    fn new_empty(tps: f32) -> Self where Self: Sized;
}

#[derive(Debug, Clone)]
pub struct MetaV1 {
    pub tps: f32,
    pub append_counter: u8,
}

impl Meta for MetaV1 {
    fn size() -> usize {
        0x40
    }

    fn tps(&self) -> f32 {
        self.tps
    }

    fn tps_dt(&self) -> f32 {
        1.0_f32 / self.tps
    }

    fn uses_dt(&self) -> bool {
        false
    }

    fn version() -> u8 {
        1
    }

    fn version_instance(&self) -> u8 {
        1
    }

    fn rng_seed(&self) -> Option<u64> {
        None
    }

    fn is_rng_seed_set(&self) -> bool {
        false
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= Self::size());
        let version = bytes[0];
        assert!(version == Self::version());
        let append_counter = bytes[1];
        let tps = f32::from_le_bytes(bytes[4..8].try_into().unwrap());

        Self {
            tps,
            append_counter,
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes = vec![0u8; Self::size()];
        bytes[0] = Self::version();
        bytes[1] = self.append_counter;
        bytes[4..8].copy_from_slice(&self.tps.to_le_bytes());
        bytes.into_boxed_slice()
    }

    fn new_empty(tps: f32) -> Self {
        Self::new(tps, 0)
    }
}

impl MetaV1 {
    pub fn new(tps: f32, append_counter: u8) -> Self {
        Self {
            tps,
            append_counter,
        }
    }
}

#[repr(u8)]
enum MetaV2BitFlags {
    OverrideSeed = 1 << 0,
    TpsInsteadOfDt = 1 << 1,
}

impl MetaV2BitFlags {
    fn is_set(flags: u8, flag: MetaV2BitFlags) -> bool {
        (flags & (flag as u8)) != 0
    }

    fn set(flags: &mut u8, flag: MetaV2BitFlags, value: bool) {
        if value {
            *flags |= flag as u8;
        } else {
            *flags &= !(flag as u8);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetaV2 {
    pub rng_seed: Option<u64>,
    tps_or_dt: f32,
    pub append_counter: u8,
    flags: u8,
}

impl Meta for MetaV2 {
    fn size() -> usize {
        0x40
    }

    fn tps(&self) -> f32 {
        if MetaV2BitFlags::is_set(self.flags, MetaV2BitFlags::TpsInsteadOfDt) {
            self.tps_or_dt
        } else {
            1.0_f32 / self.tps_or_dt
        }
    }

    fn tps_dt(&self) -> f32 {
        if MetaV2BitFlags::is_set(self.flags, MetaV2BitFlags::TpsInsteadOfDt) {
            1.0_f32 / self.tps_or_dt
        } else {
            self.tps_or_dt
        }
    }

    fn uses_dt(&self) -> bool {
        !MetaV2BitFlags::is_set(self.flags, MetaV2BitFlags::TpsInsteadOfDt)
    }

    fn version() -> u8 {
        2
    }

    fn version_instance(&self) -> u8 {
        2
    }

    fn rng_seed(&self) -> Option<u64> {
        self.rng_seed
    }

    fn is_rng_seed_set(&self) -> bool {
        MetaV2BitFlags::is_set(self.flags, MetaV2BitFlags::OverrideSeed)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= Self::size());
        let version = bytes[0];
        assert!(version == Self::version());
        let append_counter = bytes[1];
        let flags = bytes[2];
        let tps_or_dt = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let seed = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let rng_seed = if seed != 0 { Some(seed) } else { None };

        Self {
            rng_seed,
            tps_or_dt,
            append_counter,
            flags,
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes = vec![0u8; Self::size()];
        bytes[0] = Self::version();
        bytes[1] = self.append_counter;
        bytes[2] = self.flags;
        bytes[4..8].copy_from_slice(&self.tps_or_dt.to_le_bytes());
        if let Some(rng_seed) = self.rng_seed {
            bytes[8..16].copy_from_slice(&rng_seed.to_le_bytes());
        }
        bytes.into_boxed_slice()
    }

    fn new_empty(tps: f32) -> Self {
        Self::new(tps, 0, None)
    }
}

impl MetaV2 {
    pub fn new(tps: f32, append_counter: u8, rng_seed: Option<u64>) -> Self {
        let mut flags = 0;
        MetaV2BitFlags::set(&mut flags, MetaV2BitFlags::TpsInsteadOfDt, true);
        MetaV2BitFlags::set(&mut flags, MetaV2BitFlags::OverrideSeed, rng_seed.is_some());
        Self {
            rng_seed,
            tps_or_dt: tps,
            append_counter,
            flags,
        }
    }
}
