#[macro_export]
macro_rules! color {
    ($r:expr, $g:expr, $b:expr) => {
        wgpu::Color {
            r: $r,
            g: $g,
            b: $b,
            a: 1.0,
        }
    };
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        wgpu::Color {
            r: $r,
            g: $g,
            b: $b,
            a: $a,
        }
    };
}
