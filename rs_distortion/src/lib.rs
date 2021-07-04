#[macro_use]
extern crate vst;

use vst::plugin::{HostCallback, Info, Plugin, Category, PluginParameters};
use vst::util::AtomicFloat;
use vst::buffer::AudioBuffer;

use std::sync::Arc;
use std::f32;

const HALF_PI: f32 = std::f32::consts::PI / 2.0;

#[derive(Default)]
struct Distortion {
    params: Arc<DistortionParameters>,
}

struct DistortionParameters {
    // drive level
    // type - Tanh, Arctan, Chebyshev 3rd Order
    // dry/wet
    coefficient: AtomicFloat,
    distortion_type: AtomicFloat,
    level: AtomicFloat,
    dry_wet: AtomicFloat
}

impl DistortionParameters {
    fn get_distortion_name(&self) -> String {        
        match self.distortion_type.get() {
            t if t < 0.5 => "Tanh",
            t if t < 1.0 => "Arctan",
            t if t == 1.0 => "Chebyshev 3rd Order",
            _ => "" 
        }.to_string()
    }

    fn get_distortion_type_rounded(&self) -> i32 {
        match self.distortion_type.get() {
            t if t < 0.5 => 0,
            t if t < 1.0 => 1,
            t if t == 1.0 => 2,
            _ => 0 
        }
    }
}

impl Default for DistortionParameters {
    fn default() -> DistortionParameters {
        DistortionParameters {
            coefficient: AtomicFloat::new(0.5),
            distortion_type: AtomicFloat::new(0.0),
            level: AtomicFloat::new(0.5),
            dry_wet: AtomicFloat::new(0.5)
        }
    }
}

impl PluginParameters for DistortionParameters {
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", (self.coefficient.get() * 9.0) + 1.0),
            1 => self.get_distortion_name(),
            2 => format!("{:.2}", self.level.get()),
            3 => format!("{:.2}", self.dry_wet.get()),
            _ => "".to_string()
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Coefficient",
            1 => "Type",
            2 => "Level",
            3 => "Dry/Wet",
            _ => ""
        }.to_string()
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.coefficient.get(),
            1 => self.distortion_type.get(),
            2 => self.level.get(),
            3 => self.dry_wet.get(),
            _ => 0.0
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.coefficient.set(value),
            1 => self.distortion_type.set(value),
            2 => self.level.set(value),
            3 => self.dry_wet.set(value),
            _ => ()
        }
    }
}

impl Distortion {
    fn process_sample(&self, sample: f32) -> f32 {
        let level = (self.params.coefficient.get() * 9.0) + 1.0;
        let processed = match self.params.distortion_type.get() {
            t if t < 0.5 => level * sample.tanh(),
            t if t < 1.0 => level * sample.atanh() * HALF_PI,
            t if t == 1.0 => level * (4.0 * sample.powi(3) - (3.0 * sample)),
            _ => 0.0    
        };
        ((sample * (1.0 - self.params.dry_wet.get())) + (processed * self.params.dry_wet.get())) * self.params.level.get()
    }
}

impl Plugin for Distortion {
    fn get_info(&self) -> Info {
        Info {
            name: "Oxidize".to_string(),
            vendor: "Ryan Jeffares".to_string(),
            unique_id: 129153,
            version: 1,
            inputs: 2,
            outputs: 2,
            parameters: 4,
            category: Category::Effect,
            ..Default::default()
        }
    }

    fn new(_host: HostCallback) -> Self {
        Distortion {
            params: Arc::new(DistortionParameters::default())
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {        
        for (input_buffer, output_buffer) in buffer.zip() {            
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                *output_sample = self.process_sample(*input_sample);
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

plugin_main!(Distortion);