use std::{sync::Arc, collections::VecDeque};

use sdl2::{render::{Texture, Canvas, TextureCreator},video::Window, pixels::PixelFormatEnum};

use crate::{fft::{FftProcessor, Spectrum}, ring::Ring};

pub struct SpectogramTexture<'a> {
    pub texture: Texture<'a>,
    processor: FftProcessor,
    history: Ring<Spectrum>,
}

impl<'a> SpectogramTexture<'a> {
    pub fn new<T>(creator: &'a TextureCreator<T>, width: u32, height: u32, processor: FftProcessor) -> Self {
        Self {
            texture: creator.create_texture_streaming(PixelFormatEnum::RGBA32, width, height).unwrap(),
            processor,
            history: Ring::new(width.try_into().unwrap()),
        }
    }

    pub fn update(&mut self) {
        'processing: loop {
            match self.processor.poll() {
                None => break 'processing,
                Some(v) => drop(self.history.send(v)),
            }
        }

        let query = self.texture.query();
        
        self.texture.with_lock(
            None,
            | data: &mut [u8], pitch: usize | {
                for x in 0..query.width {
                    for y in 0..query.height {
                        // distance from right edge
                        let right = query.width - x - 1;
                        if right >= self.history.len() as u32 {
                            continue; // no data here
                        }

                        // the start of the spectrogram
                        let spec_start = query.width - self.history.len() as u32;

                        // how many samples past the beginning of the spectrogram are we
                        let spec_x = x - spec_start;

                        let offset = x as usize * 4 + (query.height - y - 1) as usize * pitch;
                        
                        data[offset] = (self.history.data[spec_x as usize].get_value_at_frequency(y as f32 * 2.0).unwrap_or_default().abs().clamp(0.0, 255.0)) as u8;
                    }
                }
            }
        ).unwrap();
    }
}