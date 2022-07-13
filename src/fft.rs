use std::sync::{Arc, Mutex};

use rustfft::{FftPlanner, num_complex::Complex, Fft};

use crate::ring::Ring;

pub struct FftProcessor {
    source: Arc<Mutex<Ring<f32>>>,
    window_size: usize,
    stride: usize,
    samples: Vec<f32>,
    fft: Arc<dyn Fft<f32>>
}

impl FftProcessor {
    pub fn new(source: Arc<Mutex<Ring<f32>>>, window_size: usize, stride: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(window_size);

        Self {
            source,
            window_size,
            stride,
            samples: Vec::new(),
            fft
        }
    }

    pub fn poll(&mut self) -> Option<Vec<f32>> {
        if self.samples.len() < self.window_size {
            let mut guard = self.source.lock().unwrap();

            self.samples.append(&mut guard.recv_many(self.window_size - self.samples.len()));
        }

        if self.samples.len() < self.window_size {
            None // not enough samples
        } else {
            let mut result = vec![ Complex { re: 0.0f32, im: 0.0f32 }; self.window_size ];

            for i in 0..self.window_size {
                result[i].re = self.samples[i];
            }

            self.fft.process(result.as_mut_slice());

            let mut mags = vec![0.0f32; self.window_size / 2 + 1];

            for i in 0..(self.window_size / 2 + 1) {
                mags[i] = result[i].re * result[i].re + result[i].im * result[i].im;
            }

            self.samples = self.samples.split_off(self.stride);

            return Some(mags);
        }
    }
}