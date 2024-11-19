use std::default::Default;
use std::ops::Deref;

/// A key used to identify a `Sampler`.
///
/// Meaning of the bits from the most significant to the least significant bit:
/// - [0]: Whether the sampler is a comparison sampler. If set, the sampler is a comparison sampler.
/// - [1-2]: The address mode for the u (i.e. x) direction; 0 = ClampToEdge, 1 = Repeat, 2 = MirrorRepeat, 3 = ClampToBorder.
/// - [3-4]: The address mode for the v (i.e. y) direction; 0 = ClampToEdge, 1 = Repeat, 2 = MirrorRepeat, 3 = ClampToBorder.
/// - [5-6]: The address mode for the w (i.e. z) direction; 0 = ClampToEdge, 1 = Repeat, 2 = MirrorRepeat, 3 = ClampToBorder.
/// - [7]: The mag filtert; 0 = Nearest, 1 = Linear.
/// - [8]: The min filter; 0 = Nearest, 1 = Linear.
/// - [9]: The mipmap filter; 0 = Nearest, 1 = Linear.
/// - [10-12]: The compare function; 0 = Never, 1 = Less, 2 = Equal, 3 = LessEqual, 4 = Greater, 5 = NotEqual, 6 = GreaterEqual, 7 = Always.
/// NOTE: the lod_min_clamp, lod_max_clamp, anisotropy_clamp, and border_color are not included in the key.
/// TODO: add support for the missing fields.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct SamplerId(u32);

impl Default for SamplerId {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! u32_to_wgpu_address_mode {
    ($e:expr) => {
        match $e {
            0 => wgpu::AddressMode::ClampToEdge,
            1 => wgpu::AddressMode::Repeat,
            2 => wgpu::AddressMode::MirrorRepeat,
            3 => wgpu::AddressMode::ClampToBorder,
            _ => unreachable!(),
        }
    };
}

macro_rules! u32_to_wgpu_filter_mode {
    ($e:expr) => {
        match $e {
            0 => wgpu::FilterMode::Nearest,
            1 => wgpu::FilterMode::Linear,
            _ => unreachable!(),
        }
    };
}

impl SamplerId {
    /// Creates a new `SamplerId` with an invalid value.
    pub fn new() -> Self {
        Self(u32::MAX)
    }

    /// Creates a new `SamplerId` from a `wgpu::SamplerDescriptor`.
    pub fn from_descriptor(descriptor: &wgpu::SamplerDescriptor) -> Self {
        let mut id = 0;
        id |= (descriptor.compare.is_some() as u32) << 31;
        id |= (descriptor.address_mode_u as u32) << 29;
        id |= (descriptor.address_mode_v as u32) << 27;
        id |= (descriptor.address_mode_w as u32) << 25;
        id |= (descriptor.mag_filter as u32) << 24;
        id |= (descriptor.min_filter as u32) << 23;
        id |= (descriptor.mipmap_filter as u32) << 22;
        id |= (descriptor.compare.unwrap_or(wgpu::CompareFunction::Always) as u32 - 1) << 19;
        Self(id)
    }

    /// Returns whether the `SamplerId` is invalid.
    pub fn is_invalid(&self) -> bool {
        self.0 == u32::MAX
    }

    pub fn address_mode_u(&self) -> wgpu::AddressMode {
        u32_to_wgpu_address_mode!((self.0 >> 29) & 0b11)
    }

    pub fn address_mode_v(&self) -> wgpu::AddressMode {
        u32_to_wgpu_address_mode!((self.0 >> 27) & 0b11)
    }

    pub fn address_mode_w(&self) -> wgpu::AddressMode {
        u32_to_wgpu_address_mode!((self.0 >> 25) & 0b11)
    }

    pub fn mag_filter(&self) -> wgpu::FilterMode {
        u32_to_wgpu_filter_mode!((self.0 >> 24) & 0b1)
    }

    pub fn min_filter(&self) -> wgpu::FilterMode {
        u32_to_wgpu_filter_mode!((self.0 >> 23) & 0b1)
    }

    pub fn mipmap_filter(&self) -> wgpu::FilterMode {
        u32_to_wgpu_filter_mode!((self.0 >> 22) & 0b1)
    }

    pub fn compare_func(&self) -> Option<wgpu::CompareFunction> {
        if (self.0 >> 31) & 0b1 != 1 {
            return None;
        }

        Some(match (self.0 >> 19) & 0b111 {
            0 => wgpu::CompareFunction::Never,
            1 => wgpu::CompareFunction::Less,
            2 => wgpu::CompareFunction::Equal,
            3 => wgpu::CompareFunction::LessEqual,
            4 => wgpu::CompareFunction::Greater,
            5 => wgpu::CompareFunction::NotEqual,
            6 => wgpu::CompareFunction::GreaterEqual,
            7 => wgpu::CompareFunction::Always,
            _ => unreachable!(),
        })
    }
}

/// Thin wrapper around a `wgpu::Sampler` that includes a `SamplerId`.
#[derive(Debug)]
pub struct Sampler {
    /// The sampler id.
    pub id: SamplerId,
    /// The sampler.
    pub sampler: wgpu::Sampler,
}

impl Deref for Sampler {
    type Target = wgpu::Sampler;

    fn deref(&self) -> &Self::Target {
        &self.sampler
    }
}

impl PartialEq for Sampler {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Sampler {
    /// Creates a new sampler.
    pub fn new(device: &wgpu::Device, descriptor: wgpu::SamplerDescriptor) -> Self {
        let sampler = device.create_sampler(&descriptor);
        let id = SamplerId::from_descriptor(&descriptor);
        Self { sampler, id }
    }
}
