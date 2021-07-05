#[macro_use]
extern crate vst;

use vst::plugin::{HostCallback, Info, Plugin, Category, PluginParameters, CanDo};
use vst::util::AtomicFloat;
use vst::buffer::AudioBuffer;
use vst::api::{Events, Supported};
use vst::event::Event;

use std::sync::Arc;
use std::vec::Vec;

mod adsr;
mod oscillator;
mod filter;

/*
*   4 Voice Poly Synth with switchable saw/square wave and resonant lowpass/highpass/bandpass filter
*   TODO:
*   Dual osc with individual waveform types and detune
*   Filter/Pitch envelope
*   Cross Modulation
*   Filter/freq lfo
*/

#[derive(Default)]
struct Synth {
    oscillators: Vec<oscillator::Oscillator>,
    pitch_lfo: oscillator::LFO,
    params: Arc<SynthParameters>,                
    last_played_osc_index: usize,
    filter: filter::Filter,
    sample_rate: f32
}

struct SynthParameters {
    oscillator_type: AtomicFloat,
    volume: AtomicFloat,
    attack: AtomicFloat,
    decay: AtomicFloat,
    sustain: AtomicFloat,
    release: AtomicFloat,
    filter_type: AtomicFloat,
    filter_cutoff: AtomicFloat,
    filter_resonance: AtomicFloat,
    pitch_lfo_depth: AtomicFloat,
    pitch_lfo_rate: AtomicFloat 
}

impl Default for SynthParameters {
    fn default() -> SynthParameters {
        SynthParameters {
            oscillator_type: AtomicFloat::new(0.0),
            volume: AtomicFloat::new(0.5),
            attack: AtomicFloat::new(0.0),
            decay: AtomicFloat::new(0.0),
            sustain: AtomicFloat::new(1.0),
            release: AtomicFloat::new(0.0),
            filter_type: AtomicFloat::new(0.0),
            filter_cutoff: AtomicFloat::new(1.0),
            filter_resonance: AtomicFloat::new(0.07),
            pitch_lfo_depth: AtomicFloat::new(0.0),
            pitch_lfo_rate: AtomicFloat::new(0.25)    
        }
    }
}

impl PluginParameters for SynthParameters {
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => if self.oscillator_type.get() < 0.5 { "Saw" } else { "Pulse" }.to_string(),
            1 => format!("{:.2}", self.volume.get()),
            2 => format!("{:.2}", self.attack.get().powi(2) * 10.0),
            3 => format!("{:.2}", self.decay.get().powi(2) * 10.0),
            4 => format!("{:.2}", self.sustain.get()),
            5 => format!("{:.2}", self.release.get().powi(2) * 10.0),
            6 => if self.filter_type.get() < 0.33 { "Lowpass" } else if self.filter_type.get() < 0.66 { "Bandpass" } else { "Highpass" }.to_string(),
            7 => format!("{:.2}", (self.filter_cutoff.get().powi(3) * 19980.0) + 20.0),
            8 => format!("{:.2}", (self.filter_resonance.get() * 9.9) + 0.1),
            9 => format!("{:.2}", self.pitch_lfo_depth.get().powi(2) * 100.0),
            10 => format!("{:.2}", (self.pitch_lfo_rate.get() * 19.9) + 0.1),
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
            6 => "Filter Type",
            7 => "Filter Cutoff",
            8 => "Filter Resonance",
            9 => "Pitch LFO Depth",
            10 => "Pitch LFO Rate",
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
            6 => self.filter_type.get(),
            7 => self.filter_cutoff.get(),
            8 => self.filter_resonance.get(),
            9 => self.pitch_lfo_depth.get(),
            10 => self.pitch_lfo_rate.get(),
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
            6 => self.filter_type.set(value),
            7 => self.filter_cutoff.set(value),
            8 => self.filter_resonance.set(value),
            9 => self.pitch_lfo_depth.set(value),
            10 => self.pitch_lfo_rate.set(value),
            _ => ()
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {                        
            2 => "s",
            3 => "s",            
            5 => "s",            
            7 => "Hz",            
            9 => "%",
            10 => "Hz",
            _ => ""
        }.to_string()
    }
}

impl Plugin for Synth {
    fn new(_host: HostCallback) -> Self {        
        Synth {
            oscillators: vec![oscillator::Oscillator::default(), oscillator::Oscillator::default(), oscillator::Oscillator::default(), oscillator::Oscillator::default()],
            pitch_lfo: oscillator::LFO::default(),
            params: Arc::new(SynthParameters::default()),        
            last_played_osc_index: 0,
            filter: filter::Filter::default(),
            sample_rate: 44100.0
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Oscicrate".to_string(),
            vendor: "Ryan Jeffares".to_string(),
            unique_id: 129154,
            version: 1,            
            outputs: 2,
            parameters: 11,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // i would very much like to NOT have to calculate these each buffer - need a way to call from the parameters set_parameter...
        for i in 0..4 {
            self.oscillators[i].envelope.set_params(self.params.attack.get().powi(2) * 10.0, 
                self.params.decay.get().powi(2) * 10.0, 
                self.params.sustain.get(), 
                self.params.release.get().powi(2) * 10.0);
            self.oscillators[i].set_type(if self.params.oscillator_type.get() < 0.5 { oscillator::OscillatorType::Saw } else { oscillator::OscillatorType::Square });            
        }        
        self.filter.set_params(self.params.filter_cutoff.get().powi(3), self.params.filter_resonance.get(), self.params.filter_type.get(), self.sample_rate);
        self.pitch_lfo.set_params(self.params.pitch_lfo_depth.get().powi(2), (self.params.pitch_lfo_rate.get() * 19.9) + 0.1);
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();        
        for sample in 0..samples {
            let pitch_lfo_amt = self.pitch_lfo.process();
            let mut sample_value = 0.0;
            for osc in self.oscillators.iter_mut() {
                sample_value += osc.process_with_pitch_mod(pitch_lfo_amt) * self.params.volume.get();
            }
            sample_value = self.filter.process(sample_value);
            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample] = sample_value;
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
        self.sample_rate = rate; 
        for osc in self.oscillators.iter_mut() {
            osc.set_sample_rate(self.sample_rate);
        }
        self.filter.set_sample_rate(self.sample_rate);
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
        self.oscillators[self.last_played_osc_index].note_on(note, vel); 
        self.last_played_osc_index = (self.last_played_osc_index + 1) % 4;               
    }

    fn note_off(&mut self, note: u8) {           
        for osc in self.oscillators.iter_mut() {
            if osc.get_current_note() == note {
                osc.note_off();
            }
        }
    }
}

plugin_main!(Synth);