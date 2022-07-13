use std::sync::{Arc, Mutex};

use rustfft::{FftPlanner, num_complex::Complex, Fft};

use crate::ring::Ring;

#[derive(Clone, Default)]
pub struct Spectrum {
    pub bins: Vec<f32>,
    pub window_size: usize,
    pub sample_rate: u32,
}

impl Spectrum {
    pub fn bin_width(&self) -> f32 {
        (self.sample_rate as f32) / (self.window_size as f32)
    }

    pub fn index_for_freq(&self, freq: f32) -> usize {
        (freq / self.bin_width()) as usize
    }

    pub fn freq_for_index(&self, idx: usize) -> f32 {
        self.bin_width() * (idx as f32)
    }

    pub fn get_value_at_frequency(&self, freq: f32) -> Option<f32> {
        let idx = self.index_for_freq(freq);

        if idx > self.bins.len() {
            None
        } else {
            Some(self.bins[idx])
        }
    }
}

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

    pub fn poll(&mut self) -> Option<Spectrum> {
        if self.samples.len() < self.window_size {
            let mut guard = self.source.lock().unwrap();

            self.samples.extend(guard.recv_many(self.window_size - self.samples.len()));
        }

        if self.samples.len() < self.window_size {
            None // not enough samples
        } else {
            let mut result = vec![ Complex { re: 0.0f32, im: 0.0f32 }; self.window_size ];

            for i in 0..self.window_size {
                result[i].re = self.samples[i];
            }

            self.fft.process(result.as_mut_slice());

            let mut mags = vec![0.0f32; self.window_size / 2 - 1];

            for i in 0..(self.window_size / 2 - 1) {
                mags[i] = result[i].re * result[i].re + result[i].im * result[i].im;
            }

            self.samples = self.samples.split_off(self.stride);

            Some(Spectrum {
                bins: mags,
                sample_rate: 44100, // TODO: no hardcode
                window_size: self.window_size,
            })
        }
    }
}