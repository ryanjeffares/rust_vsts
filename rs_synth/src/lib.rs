#[macro_use]
extern crate vst;

use vst::plugin::{HostCallback, Info, Plugin, Category, PluginParameters, CanDo};
use vst::util::AtomicFloat;
use vst::buffer::AudioBuffer;
use vst::api::{Events, Supported};
use vst::event::Event;

use std::sync::Arc;

mod adsr;

fn mtof(note: u8) -> f32 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f32 = 440.0;
    ((f32::from(note as i8 - A4_PITCH)) / 12.0).exp2() * A4_FREQ
}

#[derive(Default)]
struct Synth {
    params: Arc<SynthParameters>,
    envelope: adsr::ADSR,
    phase: f32,
    frequency: f32,
    output: f32,
    sample_rate: f64,
    note_on: bool
}

struct SynthParameters {
    saw_volume: AtomicFloat,
    pulse_volume: AtomicFloat
}

impl Default for SynthParameters {
    fn default() -> SynthParameters {
        SynthParameters {
            saw_volume: AtomicFloat::new(0.5),
            pulse_volume: AtomicFloat::new(0.5)
        }
    }
}

impl PluginParameters for SynthParameters {
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", self.saw_volume.get()),
            1 => format!("{:.2}", self.pulse_volume.get()),
            _ => "".to_string()
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Saw Volume",
            1 => "Pulse Volume",
            _ => ""
        }.to_string()
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.saw_volume.get(),
            1 => self.pulse_volume.get(),
            _ => 0.0
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.saw_volume.set(value),
            1 => self.pulse_volume.set(value),
            _ => ()
        }
    }
}

impl Plugin for Synth {
    fn new(_host: HostCallback) -> Self {        
        Synth {
            params: Arc::new(SynthParameters::default()),
            envelope: adsr::ADSR::default(),
            phase: 0.0,
            frequency: 100.0,
            output: 0.0,
            sample_rate: 44100.0,
            note_on: false
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Oscicrate".to_string(),
            vendor: "Ryan Jeffares".to_string(),
            unique_id: 129154,
            version: 1,
            inputs: 0,
            outputs: 2,
            parameters: 2,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        for sample in 0..samples {
            let saw_value = self.saw() * self.params.saw_volume.get();
            //let square_value = self.square() * self.params.pulse_volume.get();
            //let adsr_val = self.envelope.get_value();
            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample] = (saw_value) * if self.note_on {1.0} else {0.0};// * adsr_val;
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = f64::from(rate);
        self.envelope.set_sample_rate(self.sample_rate);
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi_event(ev.data),
                _ => ()
            }
        }
    }
}

impl Synth {
    fn saw(&mut self) -> f32 {
        self.output = self.phase;
        if self.phase >= 1.0 {
            self.phase -= 2.0;
        }
        self.phase += (1.0 / (self.sample_rate as f32 / self.frequency)) * 2.0;
        self.output
    }

    fn square(&mut self) -> f32 {
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        self.phase += (1.0 / (self.sample_rate as f32 / self.frequency)) * 2.0;
        if self.phase > 0.5 { 1.0 } else { -1.0 }
    }

    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_off(data[1]),
            144 => self.note_on(data[1]),
            _ => ()
        }
    }

    fn note_on(&mut self, note: u8) {
        self.frequency = mtof(note);
        //self.envelope.start_note();
        self.note_on = true;
    }

    fn note_off(&mut self, note: u8) {
        //self.envelope.end_note();
        self.note_on = false;
    }
}

plugin_main!(Synth);