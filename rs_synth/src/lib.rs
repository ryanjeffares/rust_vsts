#[macro_use]
extern crate vst;

use vst::plugin::{HostCallback, Info, Plugin, Category, PluginParameters, CanDo};
use vst::util::AtomicFloat;
use vst::buffer::AudioBuffer;
use vst::api::{Events, Supported};
use vst::event::Event;

use std::sync::Arc;

mod adsr;
mod oscillator;

#[derive(Default)]
struct Synth {
    oscillators: [oscillator::Oscillator; 4],
    params: Arc<SynthParameters>,            
    current_velocities: [f32; 4],
    last_played_osc_index: usize
}

struct SynthParameters {
    oscillator_type: AtomicFloat,
    volume: AtomicFloat,
    attack: AtomicFloat,
    decay: AtomicFloat,
    sustain: AtomicFloat,
    release: AtomicFloat    
}

impl Default for SynthParameters {
    fn default() -> SynthParameters {
        SynthParameters {
            oscillator_type: AtomicFloat::new(0.0),
            volume: AtomicFloat::new(0.5),
            attack: AtomicFloat::new(0.0),
            decay: AtomicFloat::new(0.0),
            sustain: AtomicFloat::new(1.0),
            release: AtomicFloat::new(0.0)            
        }
    }
}

impl PluginParameters for SynthParameters {
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => if self.oscillator_type.get() < 0.5 { "Saw".to_string() } else { "Pulse".to_string() },
            1 => format!("{:.2}", self.volume.get()),
            2 => format!("{:.2}", self.attack.get() * 10.0),
            3 => format!("{:.2}", self.decay.get() * 10.0),
            4 => format!("{:.2}", self.sustain.get()),
            5 => format!("{:.2}", self.release.get() * 10.0),
            _ => "".to_string()
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Oscillator Type",
            1 => "Volume",
            2 => "Envelope Attack",
            3 => "Envelope Decay",
            4 => "Envelope Sustain",
            5 => "Envelope Release",
            _ => ""
        }.to_string()
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.oscillator_type.get(),
            1 => self.volume.get(),
            2 => self.attack.get(),
            3 => self.decay.get(),
            4 => self.sustain.get(),
            5 => self.release.get(),
            _ => 0.0
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.oscillator_type.set(value),
            1 => self.volume.set(value),
            2 => self.attack.set(value),
            3 => self.decay.set(value),
            4 => self.sustain.set(value),
            5 => self.release.set(value),
            _ => ()
        }
    }
}

impl Plugin for Synth {
    fn new(_host: HostCallback) -> Self {        
        Synth {
            oscillators: [oscillator::Oscillator::new(); 4],
            params: Arc::new(SynthParameters::default()),                      
            current_velocities: [0.0; 4],
            last_played_osc_index: 0     
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
            parameters: 6,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // i would very much like to NOT have to calculate these each buffer - need a way to call from the parameters set_parameter...
        for i in 0..4 {
            self.oscillators[i].envelope.set_params(self.params.attack.get() * 10.0, 
                self.params.decay.get() * 10.0, 
                self.params.sustain.get(), 
                self.params.release.get() * 10.0);
            self.oscillators[i].set_type(if self.params.oscillator_type.get() < 0.5 { oscillator::OscillatorType::Saw } else { oscillator::OscillatorType::Square });
        }
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();        
        for sample in 0..samples {
            let mut sample_value = 0.0;
            for i in 0..4 {
                sample_value += self.oscillators[i].process() * self.current_velocities[i];
            }            
            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample] = sample_value * self.params.volume.get();
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
        for i in 0..4 {
            self.oscillators[i].set_sample_rate(rate);
        }
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

    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_off(data[1]),
            144 => self.note_on(data[1], data[2]),
            _ => ()
        }
    }

    fn note_on(&mut self, note: u8, vel: u8) {                
        self.current_velocities[self.last_played_osc_index] = f32::from(vel) / 128.0;        
        self.oscillators[self.last_played_osc_index].note_on(note); 
        self.last_played_osc_index = (self.last_played_osc_index + 1) % 4;               
    }

    fn note_off(&mut self, note: u8) {           
        for i in 0..4 {
            if self.oscillators[i].get_current_note() == note {
                self.oscillators[i].note_off();                
            }
        }
    }
}

plugin_main!(Synth);