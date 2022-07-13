use std::sync::Arc;

use sdl2::{render::{Texture, Canvas, TextureCreator},video::Window, pixels::PixelFormatEnum};

use crate::fft::FftProcessor;

pub struct SpectogramTexture<'a> {
    pub texture: Texture<'a>,
    processor: FftProcessor
}

impl<'a> SpectogramTexture<'a> {
    pub fn new<T>(creator: &'a TextureCreator<T>, width: u32, height: u32, processor: FftProcessor) -> Self {
        Self {
            texture: creator.create_texture_streaming(PixelFormatEnum::RGBA32, width, height).unwrap(),
            processor,
        }
    }

    pub fn update(&mut self) -> bool {
        match self.processor.poll() {
            None => false,
            Some(v) => {
                let query = self.texture.query();
                
                self.texture.with_lock(
                    None,
                    | data: &mut [u8], pitch: usize | {
                        for x in 0..1 {//query.width {
                            for y in 0..query.height {
                                let offset = x as usize * 4 + y as usize * pitch;

                                data[offset] = (v[y as usize].abs().log2() + 64.0) as u8;
                            }
                        }
                    }
                ).unwrap();

                true
            }
        }
    }
}