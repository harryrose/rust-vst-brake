#[macro_use]
extern crate vst;

use std::sync::Arc;
use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

const NO_CHANNELS: i32 = 2;
const BUF_LEN_SECS: usize = 10;

type SampleType = f32;

struct Braker {
    buf: Box<[(SampleType, SampleType)]>,
    play_head: f32,
    rec_head: usize,
    speed: f32,
    params: Arc<BrakerParameters>,
}

struct BrakerParameters {
    brake: AtomicFloat,
    brake_speed: AtomicFloat,
}

impl Default for Braker {
    fn default() -> Braker {
        Braker {
            buf: Box::<[(SampleType, SampleType)]>::default(),
            params: Arc::new(BrakerParameters::default()),
            play_head: 0.0,
            speed: 1.0,
            rec_head: 0,
        }
    }
}

impl Plugin for Braker {
    fn get_info(&self) -> Info {
        Info {
            name: "Braker".to_string(),
            unique_id: 18423,
            category: Category::Effect,
            version: 1,
            inputs: NO_CHANNELS,
            outputs: NO_CHANNELS,
            parameters: 2,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        let buf = vec![(0 as SampleType, 0 as SampleType); rate as usize * BUF_LEN_SECS];
        self.buf = buf.into_boxed_slice();
        self.rec_head = 0;
        self.play_head = 0.0;
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        if self.params.brake.get() <= 0.5 {
            self.play_head = 0.;
            self.rec_head = 0;
            self.speed = 1.;
            for (input_buffer, output_buffer) in buffer.zip() {
                for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                    *output_sample = *input_sample;
                }
            }
            return;
        }

        let (inputs, outputs) = buffer.split();
        let (l, r) = inputs.split_at(1);
        for (l, r) in l[0].iter().zip(r[0].iter()) {
            if self.rec_head >= self.buf.len() {
                break;
            }
            self.buf[self.rec_head] = (*l, *r);
            self.rec_head += 1;
        }

        let (mut l, mut r) = outputs.split_at_mut(1);
        for (l, r) in l[0].iter_mut().zip(r[0].iter_mut()) {
            let sample = if self.speed < 0.01 || self.play_head >= (self.rec_head as f32 - 1.0) {
                (0., 0.)
            } else {
                let flr = self.play_head.floor() as usize;
                let nxt = flr + 1;
                let diff = self.play_head - flr as f32;
                let (tl, tr) = self.buf[flr];
                let (nl, nr) = self.buf[nxt];
                (tl + (nl - tl) * diff, tr + (nr - tr) * diff)
            };
            self.play_head += self.speed;
            self.speed *= 0.999 + (0.5 + self.params.brake_speed.get() / 2.) / 1000.;
            *l = sample.0;
            *r = sample.1;
        }
    }
}

impl Default for BrakerParameters {
    fn default() -> BrakerParameters {
        BrakerParameters {
            brake: AtomicFloat::new(0.0),
            brake_speed: AtomicFloat::new(0.9999),
        }
    }
}

impl PluginParameters for BrakerParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.brake.get(),
            1 => self.brake_speed.get(),
            _ => 0.0,
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "brake",
            1 => "brake rate",
            _ => "",
        }
        .to_string()
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!(
                "{}",
                if self.brake.get() > 0.5 {
                    "enabled"
                } else {
                    "disabled"
                }
            ),
            1 => format!("{:.2}", self.brake_speed.get()),
            _ => "".to_string(),
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.brake.set(if value > 0.5 { 1.0 } else { 0.0 }),
            1 => self.brake_speed.set(if value < 0.0 {
                0.0
            } else if value > 1.0 {
                1.0
            } else {
                value
            }),
            _ => (),
        }
    }
}
plugin_main!(Braker);
