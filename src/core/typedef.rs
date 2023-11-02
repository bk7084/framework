//! Type definitions for commonly used structures across the library.

/// Hash map for small keys.
pub type FxHashMap<K, V> = rustc_hash::FxHashMap<K, V>;

/// Hash set for small keys.
pub type FxHashSet<K> = rustc_hash::FxHashSet<K>;

/// Hasher for small keys.
pub type FxHasher = rustc_hash::FxHasher;

/// Build hasher for small keys.
pub type FxBuildHasher = std::hash::BuildHasherDefault<FxHasher>;

/// A string optimized for small strings shorter than 23 characters.
pub type SmlString = smartstring::SmartString<smartstring::LazyCompact>;

/// A vector with fixed capacity.
pub type ArrVec<T, const N: usize> = arrayvec::ArrayVec<T, N>;

/// Customized format macro generating a [`SmlString`].
#[macro_export]
macro_rules! format_sml {
    ($($arg:tt)*) => {{
        use std::fmt::Write as _;
        let mut buffer = $crate::util::typedefs::SmlString::new();
        write!(buffer, $($arg)*).expect("unexpected formatting error");
        buffer
    }};
}
