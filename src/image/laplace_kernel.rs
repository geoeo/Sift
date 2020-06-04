use crate::Float;
use super::kernel::Kernel;


pub struct LaplaceKernel {
    kernel: Vec<Float>,
    half_repeat: usize

}

impl LaplaceKernel {


    pub fn new(half_repeat: usize) -> LaplaceKernel {
        LaplaceKernel {
            kernel: vec![1.0,-2.0,1.0],
            half_repeat
        }
    }
}

impl Kernel for LaplaceKernel {
    fn kernel(&self) -> &Vec<Float> {
        &self.kernel
    }
    fn step(&self) -> usize {
        1
    }
    fn half_repeat(&self) -> usize {
        self.half_repeat
    }
}