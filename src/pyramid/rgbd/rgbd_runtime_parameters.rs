use crate::Float;

#[derive(Debug,Clone)]
pub struct RGBDRuntimeParameters {
    pub sigma: Float,
    pub blur_radius: Float,
    pub octave_count: usize,
    pub min_image_dimensions: (usize,usize)
}