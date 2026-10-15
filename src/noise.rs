#![allow(clippy::approx_constant, clippy::excessive_precision)]

pub const SIDE: usize = 1024;
const LUT_SIZE: usize = 4096;

pub(crate) struct Noise {
    sine: Box<[f32; LUT_SIZE]>,
}

impl Noise {
    pub(crate) fn new() -> Self {
        let mut sine = Box::new([0.0; LUT_SIZE]);
        for (i, value) in sine.iter_mut().enumerate() {
            // Preserve the recovered f32 expression. Linux sinf is a substitute
            // for the Windows helper, not an established bit-exact equivalent.
            *value = (i as f32 * 6.283185 * 0.0009765625 * 4.0).sin();
        }
        Self { sine }
    }

    #[inline(never)]
    fn gradient(&self, x: f32, y: f32) -> (f32, f32) {
        let sample = |phase: f32| {
            let mut fraction = phase - phase.trunc();
            if fraction < 0.0 {
                fraction += 1.0;
            }
            // Adding 1 can round a tiny negative fraction to 1 in f32.
            // Clamp the index instead of permitting an out-of-bounds read.
            let index = ((fraction * 1024.0 * 4.0) as usize).min(LUT_SIZE - 1);
            let value = self.sine[index] * 43758.547;
            (value - value.trunc()) * 2.0 - 1.0
        };
        (
            sample((y * 311.7 + x * 127.1) * 0.15915495),
            sample((y * 183.3 + x * 269.5) * 0.15915495),
        )
    }

    #[inline(never)]
    fn at(&self, x: f32, y: f32) -> f32 {
        let skew = (x + y) * 0.36602542;
        let floor = |v: f32| {
            let i = v as i32;
            (i - i32::from(v < i as f32)) as f32
        };
        let ix = floor(skew + x);
        let iy = floor(skew + y);
        let unskew = (iy + ix) * 0.21132487;
        let dx0 = (x - ix) + unskew;
        let dy0 = (y - iy) + unskew;
        let (step_x, step_y) = if dx0 > dy0 { (1.0, 0.0) } else { (0.0, 1.0) };
        let dx1 = (dx0 - step_x) + 0.21132487;
        let dy1 = (dy0 - step_y) + 0.21132487;
        let dx2 = (dx0 - 1.0) + 0.42264974;
        let dy2 = (dy0 - 1.0) + 0.42264974;
        let t0 = (0.5 - (dy0 * dy0 + dx0 * dx0)).max(0.0);
        let t1 = (0.5 - (dy1 * dy1 + dx1 * dx1)).max(0.0);
        let t2 = (0.5 - (dy2 * dy2 + dx2 * dx2)).max(0.0);
        let g0 = self.gradient(ix, iy);
        let g1 = self.gradient(ix + step_x, iy + step_y);
        let g2 = self.gradient(ix + 1.0, iy + 1.0);
        t1 * t1 * t1 * t1 * (g1.1 * dy1 + g1.0 * dx1)
            + t0 * t0 * t0 * t0 * (g0.1 * dy0 + g0.0 * dx0)
            + (g2.1 * dy2 + g2.0 * dx2) * t2 * t2 * t2 * t2
    }

    #[inline(never)]
    fn terrain(&self, x: i32, y: i32) -> f32 {
        let start_x = x as f32 * 1.1 * 0.0009765625;
        let start_y = y as f32 * 1.1 * 0.0009765625;
        let (mut x, mut y) = (start_x, start_y);
        let (mut ridge, mut amplitude) = (0.0, 0.8);
        for _ in 0..10 {
            ridge += (self.at(x, y) * amplitude * 40.0).abs();
            let old_x = x;
            x = x * 1.6 - y * 1.2;
            y = old_x * 1.2 + y * 1.6;
            amplitude *= 0.7;
        }
        let (mut x, mut y) = (start_x, start_y);
        let (mut signed, mut amplitude) = (0.0, 0.7);
        for _ in 0..10 {
            signed += self.at(x, y) * amplitude * 40.0;
            let old_x = x;
            x = x * 1.6 - y * 1.2;
            y = old_x * 1.2 + y * 1.6;
            amplitude *= 0.6;
        }
        (signed + ridge) * signed * 8.0 * ridge + 0.2
    }

    #[inline(never)]
    fn detail(&self, x: i32, y: i32) -> f32 {
        let (mut px, mut py) = (x as f32 * 2.2 * 0.0009765625, y as f32 * 2.2 * 0.0009765625);
        let (mut signed, mut amplitude) = (0.0, 0.4);
        for _ in 0..7 {
            signed += self.at(px, py) * amplitude * 40.0;
            let old_x = px;
            px = px * 1.6 - py * 1.2;
            py = old_x * 1.2 + py * 1.6;
            amplitude *= 0.6;
        }
        let (mut px, mut py) = (
            x as f32 * 3.3000002 * 0.0009765625,
            y as f32 * 3.3000002 * 0.0009765625,
        );
        let (mut ridge, mut amplitude) = (0.0, 0.4);
        for _ in 0..7 {
            ridge += (self.at(px, py) * amplitude * 40.0).abs();
            let old_x = px;
            px = px * 1.6 - py * 1.2;
            py = old_x * 1.2 + py * 1.6;
            amplitude *= 0.6;
        }
        ridge + signed
    }

    fn pixel(&self, x: i32, y: i32) -> [u8; 4] {
        let terrain = self.terrain(x, y);
        let detail = self.detail(x, y);
        let base = (detail * 0.6 + 0.5).clamp(0.0, 1.0);
        let mix = (detail + terrain).clamp(0.0, 1.0);
        let inverse = 1.0 - mix;
        [
            (((base + 0.1).clamp(0.0, 1.0) * mix + inverse * 0.2) * 255.0) as u8,
            (((base + 0.2).clamp(0.0, 1.0) * mix + inverse * 0.4) * 255.0) as u8,
            (((base * 0.9 + 0.3).clamp(0.0, 1.0) * mix + inverse * 0.6) * 255.0) as u8,
            255,
        ]
    }

    #[inline(never)]
    pub(crate) fn row(&self, pixels: &mut [u8], frame: i32, row: i32) {
        for (x, pixel) in pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            pixel.copy_from_slice(&self.pixel(frame + x as i32, frame + row));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lattice_origin_and_analytic_pixel() {
        let noise = Noise::new();
        assert_eq!(noise.at(0.0, 0.0), 0.0);
        // Derived analytically from the origin and color equations, NOT a
        // reference vector obtained by executing the Windows program.
        assert_eq!(noise.pixel(0, 0), [71, 117, 160, 255]);
    }

    #[test]
    fn negative_coordinates_and_frame_offsets_are_finite_and_repeatable() {
        let noise = Noise::new();
        for (x, y) in [(-4096, 17), (-1, -1), (1024, 2048), (1800, 2300)] {
            assert!(noise.at(x as f32, y as f32).is_finite());
            assert!(noise.terrain(x, y).is_finite());
            assert!(noise.detail(x, y).is_finite());
        }
        let mut row = vec![0; SIDE * 4];
        noise.row(&mut row, 3, 7);
        assert_eq!(&row[..4], &noise.pixel(3, 10));
        assert_eq!(&row[SIDE * 4 - 4..], &noise.pixel(1026, 10));
        assert!(row.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 255));
        assert!(
            row.as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[..3] != row[..3])
        );
    }
}
