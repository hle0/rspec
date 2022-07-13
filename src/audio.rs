use std::sync::{Arc, Mutex};

use cpal::{traits::{HostTrait, DeviceTrait}, SampleRate};

use crate::ring::Ring;

pub struct AudioStream {
    buffer: Arc<Mutex<Ring<f32>>>,
    stream: cpal::Stream
}

impl AudioStream {
    pub fn new(buffer: Arc<Mutex<Ring<f32>>>) -> Self {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("no input device available");
        let mut supported_configs_range = device.supported_output_configs().expect("error while querying configs");
        let supported_config = supported_configs_range.next().expect("no supported config").with_sample_rate(SampleRate(44100));

        Self {
            buffer: buffer.clone(),
            stream: {
                device.build_input_stream(
                    &supported_config.config(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let dropped = {
                            let mut lock = buffer.lock().unwrap();
            
                            lock.send_many(&mut data.to_vec())
                        };

                        if dropped > 0 {
                            eprintln!("warning: dropping {} samples, fft can't keep up!", dropped)
                        }
                    },
                    move |err| {
                        eprintln!("{}", err);
                    }
                ).unwrap()
            }
        }
        
    }
}