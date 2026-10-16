//! Core, platform-independent pixel canvas for RustPaint95.
//! This is an initial foundation, not yet verified against Windows 95 output.
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }
impl Rgb {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot { width: u32, height: u32, pixels: Vec<Rgb> }

/// A single opaque bitmap, deliberately without layers or transparency.
pub struct Canvas {
    current: Snapshot,
    undo: VecDeque<Snapshot>,
    redo: VecDeque<Snapshot>,
}
impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self, &'static str> {
        if width == 0 || height == 0 { return Err("empty canvas"); }
        let len = (width as usize).checked_mul(height as usize).ok_or("canvas too large")?;
        Ok(Self {
            current: Snapshot { width, height, pixels: vec![Rgb::WHITE; len] },
            undo: VecDeque::new(), redo: VecDeque::new(),
        })
    }

    pub fn dimensions(&self) -> (u32, u32) { (self.current.width, self.current.height) }

    pub fn pixel(&self, x: i32, y: i32) -> Option<Rgb> {
        self.index(x, y).map(|i| self.current.pixels[i])
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x as u32 >= self.current.width || y as u32 >= self.current.height {
            None
        } else {
            Some(y as usize * self.current.width as usize + x as usize)
        }
    }

    pub fn begin_action(&mut self) {
        // One complete mouse gesture is one undoable action.
        if self.undo.len() == 3 { self.undo.pop_front(); }
        self.undo.push_back(self.current.clone());
        self.redo.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(previous) = self.undo.pop_back() {
            if self.redo.len() == 3 { self.redo.pop_front(); }
            self.redo.push_back(std::mem::replace(&mut self.current, previous));
            true
        } else { false }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo.pop_back() {
            if self.undo.len() == 3 { self.undo.pop_front(); }
            self.undo.push_back(std::mem::replace(&mut self.current, next));
            true
        } else { false }
    }

    /// Draw a pixel. Call begin_action once before a mouse gesture, not per pixel.
    pub fn put_pixel(&mut self, x: i32, y: i32, color: Rgb) {
        if let Some(i) = self.index(x, y) { self.current.pixels[i] = color; }
    }

    /// Integer Bresenham line; exact Windows 95 raster behavior still needs comparison.
    pub fn line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Rgb) {
        let dx = (i64::from(x1) - i64::from(x0)).abs();
        let sx = if x0 < x1 { 1i64 } else { -1 };
        let dy = -(i64::from(y1) - i64::from(y0)).abs();
        let sy = if y0 < y1 { 1i64 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.put_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x0 = (i64::from(x0) + sx) as i32; }
            if e2 <= dx { err += dx; y0 = (i64::from(y0) + sy) as i32; }
        }
    }

    /// A 24-bit uncompressed BMP. More period-correct modes follow in later milestones.
    pub fn to_bmp24(&self) -> Result<Vec<u8>, &'static str> {
        let w = self.current.width as usize;
        let h = self.current.height as usize;
        let row = w.checked_mul(3).and_then(|x| x.checked_add(3))
            .map(|x| x & !3).ok_or("image too large")?;
        let payload = row.checked_mul(h).ok_or("image too large")?;
        let size = 54usize.checked_add(payload).ok_or("image too large")?;
        let size32 = u32::try_from(size).map_err(|_| "BMP exceeds 4 GiB")?;
        let width = i32::try_from(w).map_err(|_| "BMP width too large")?;
        let height = i32::try_from(h).map_err(|_| "BMP height too large")?;
        let mut bmp = Vec::with_capacity(size);
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&size32.to_le_bytes());
        bmp.extend_from_slice(&[0; 4]);
        bmp.extend_from_slice(&54u32.to_le_bytes());
        bmp.extend_from_slice(&40u32.to_le_bytes());
        bmp.extend_from_slice(&width.to_le_bytes());
        bmp.extend_from_slice(&height.to_le_bytes());
        bmp.extend_from_slice(&1u16.to_le_bytes());
        bmp.extend_from_slice(&24u16.to_le_bytes());
        bmp.extend_from_slice(&0u32.to_le_bytes());
        bmp.extend_from_slice(&(payload as u32).to_le_bytes());
        bmp.extend_from_slice(&[0; 16]);
        let padding = row - w * 3;
        for y in (0..h).rev() {
            for pixel in &self.current.pixels[y * w..(y + 1) * w] {
                bmp.extend_from_slice(&[pixel.b, pixel.g, pixel.r]);
            }
            bmp.extend(std::iter::repeat(0).take(padding));
        }
        Ok(bmp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn undo_is_limited_to_three_actions() {
        let mut c = Canvas::new(4, 4).unwrap();
        for i in 0..4 {
            c.begin_action();
            c.put_pixel(i, 0, Rgb::BLACK);
        }
        assert!(c.undo()); assert!(c.undo()); assert!(c.undo());
        assert!(!c.undo());
        assert_eq!(c.pixel(0, 0), Some(Rgb::BLACK));
        assert_eq!(c.pixel(1, 0), Some(Rgb::WHITE));
    }
    #[test]
    fn bmp_is_bottom_up_bgr_and_four_byte_aligned() {
        let mut c = Canvas::new(1, 2).unwrap();
        c.put_pixel(0, 0, Rgb {r: 1, g: 2, b: 3});
        c.put_pixel(0, 1, Rgb {r: 4, g: 5, b: 6});
        let bmp = c.to_bmp24().unwrap();
        assert_eq!(&bmp[..2], b"BM");
        assert_eq!(bmp.len(), 62);
        assert_eq!(&bmp[54..58], &[6, 5, 4, 0]);
        assert_eq!(&bmp[58..62], &[3, 2, 1, 0]);
    }
}
