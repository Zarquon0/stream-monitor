#[cfg(feature = "timing")]
use std::time::Instant;

#[cfg(feature = "timing")]
pub struct Timer {
    start: Instant,
    label: &'static str,
}

#[cfg(feature = "timing")]
impl Timer {
    pub fn new(label: &'static str) -> Self {
        Self {
            start: Instant::now(),
            label,
        }
    }
}

#[cfg(feature = "timing")]
impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        println!("⏱️ {} took {:?}", self.label, elapsed);
    }
}

//Blank Timer implementation when timing is not enabled

#[cfg(not(feature = "timing"))]
pub struct Timer;

#[cfg(not(feature = "timing"))]
impl Timer {
    #[inline(always)]
    pub fn new(_: &'static str) -> Self { Self }
}