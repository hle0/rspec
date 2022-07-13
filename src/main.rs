use std::{time::Duration, sync::{Arc, Mutex}};
use fft::FftProcessor;
use sdl2::event::Event;

mod ring;
mod audio;
mod fft;
mod spectrogram;

fn main() {
    let the_ring = Arc::from(
        Mutex::from(
            ring::Ring::new(8192)
        )
    );

    let audio_stream = audio::AudioStream::new(the_ring.clone());

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rspec", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    
    let mut canvas = window.into_canvas().build().unwrap();
    let mut creator = canvas.texture_creator();

    let mut spec = spectrogram::SpectogramTexture::new(
        &creator,
        1,
        480,
        FftProcessor::new(the_ring.clone(), 2048, 40)
    );

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                _ => {}
            }
        }

        canvas.clear();
        spec.update();
        canvas.copy(&spec.texture, None, None).unwrap();
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000 / 60))
    }
}
