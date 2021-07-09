#[macro_use]
extern crate vst;

use oscillator::OscillatorType;
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
    oscillators_one: Vec<oscillator::Oscillator>,    
    oscillators_two: Vec<oscillator::Oscillator>,    
    pitch_lfo: oscillator::LFO,
    params: Arc<SynthParameters>,                
    last_played_osc_index: usize,
    filters: Vec<filter::Filter>,    
    sample_rate: f32
}

struct SynthParameters {
    oscillator_one_type: AtomicFloat,
    oscillator_one_pulsewidth: AtomicFloat,
    oscillator_one_octave: AtomicFloat,
    oscillator_two_type: AtomicFloat,
    oscillator_two_pulsewidth: AtomicFloat,
    oscillator_two_octave: AtomicFloat,
    volume: AtomicFloat,
    attack: AtomicFloat,
    decay: AtomicFloat,
    sustain: AtomicFloat,
    release: AtomicFloat,
    filter_type: AtomicFloat,
    filter_cutoff: AtomicFloat,
    filter_resonance: AtomicFloat,
    filter_attack: AtomicFloat,
    filter_decay: AtomicFloat,
    filter_sustain: AtomicFloat,
    filter_release: AtomicFloat,
    pitch_lfo_depth: AtomicFloat,
    pitch_lfo_rate: AtomicFloat 
}

impl Default for SynthParameters {
    fn default() -> SynthParameters {
        SynthParameters {
            oscillator_one_type: AtomicFloat::new(0.0),
            oscillator_one_pulsewidth: AtomicFloat::new(0.5),
            oscillator_one_octave: AtomicFloat::new(0.5),
            oscillator_two_type: AtomicFloat::new(0.0),
            oscillator_two_pulsewidth: AtomicFloat::new(0.5),
            oscillator_two_octave: AtomicFloat::new(0.5),
            volume: AtomicFloat::new(0.5),
            attack: AtomicFloat::new(0.0),
            decay: AtomicFloat::new(0.0),
            sustain: AtomicFloat::new(1.0),
            release: AtomicFloat::new(0.0),
            filter_type: AtomicFloat::new(0.0),
            filter_cutoff: AtomicFloat::new(1.0),
            filter_resonance: AtomicFloat::new(0.07),
            filter_attack: AtomicFloat::new(0.0),
            filter_decay: AtomicFloat::new(0.0),
            filter_sustain: AtomicFloat::new(1.0),
            filter_release: AtomicFloat::new(0.0),
            pitch_lfo_depth: AtomicFloat::new(0.0),
            pitch_lfo_rate: AtomicFloat::new(0.25)    
        }
    }
}

impl PluginParameters for SynthParameters {
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => match self.oscillator_one_type.get() {
                t if t < 0.25 => "Saw",
                t if t < 0.5 => "Pulse",
                t if t < 0.75 => "Tri",
                _ => "Sine"
            }.to_string(),
            1 => format!("{:.2}", self.oscillator_one_pulsewidth.get()),
            2 => match self.oscillator_one_octave.get() {
                o if o < 0.2 => "-2",
                o if o < 0.4 => "-1",
                o if o < 0.6 => "0",
                o if o < 0.8 => "+1",
                _ => "+2"
            }.to_string(),
            3 => match self.oscillator_two_type.get() {
                t if t < 0.25 => "Saw",
                t if t < 0.5 => "Pulse",
                t if t < 0.75 => "Tri",
                _ => "Sine"
            }.to_string(),
            4 => format!("{:.2}", self.oscillator_two_pulsewidth.get()),
            5 => match self.oscillator_two_octave.get() {
                o if o < 0.2 => "-2",
                o if o < 0.4 => "-1",
                o if o < 0.6 => "0",
                o if o < 0.8 => "+1",
                _ => "+2"
            }.to_string(),
            6 => format!("{:.2}", self.volume.get()),
            7 => format!("{:.2}", self.attack.get().powi(2) * 10.0),
            8 => format!("{:.2}", self.decay.get().powi(2) * 10.0),
            9 => format!("{:.2}", self.sustain.get()),
            10 => format!("{:.2}", self.release.get().powi(2) * 10.0),            
            11 => if self.filter_type.get() < 0.33 { "Lowpass" } else if self.filter_type.get() < 0.66 { "Bandpass" } else { "Highpass" }.to_string(),
            12 => format!("{:.2}", (self.filter_cutoff.get().powi(3) * 19980.0) + 20.0),
            13 => format!("{:.2}", (self.filter_resonance.get() * 9.9) + 0.1),
            14 => format!("{:.2}", self.filter_attack.get().powi(2) * 10.0),
            15 => format!("{:.2}", self.filter_decay.get().powi(2) * 10.0),
            16 => format!("{:.2}", self.filter_sustain.get()),
            17 => format!("{:.2}", self.filter_release.get().powi(2) * 10.0),
            18 => format!("{:.2}", self.pitch_lfo_depth.get().powi(2) * 100.0),
            19 => format!("{:.2}", (self.pitch_lfo_rate.get() * 19.9) + 0.1),
            _ => "".to_string()
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Oscillator 1 Type",
            1 => "Oscillator 1 Pulsewidth",
            2 => "Oscillator 1 Octave",
            3 => "Oscillator 2 Type",
            4 => "Oscillator 2 Pulsewidth",
            5 => "Oscillator 2 Octave",
            6 => "Volume",
            7 => "Envelope Attack",
            8 => "Envelope Decay",
            9 => "Envelope Sustain",
            10 => "Envelope Release",            
            11 => "Filter Type",
            12 => "Filter Cutoff",
            13 => "Filter Resonance",
            14 => "Filter Attack",
            15 => "Filter Decay",
            16 => "Filter Sustain",
            17 => "Filter Release",
            18 => "Pitch LFO Depth",
            19 => "Pitch LFO Rate",
            _ => ""
        }.to_string()
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.oscillator_one_type.get(),
            1 => self.oscillator_one_pulsewidth.get(),
            2 => self.oscillator_one_octave.get(),
            3 => self.oscillator_two_type.get(),
            4 => self.oscillator_two_pulsewidth.get(),
            5 => self.oscillator_two_octave.get(),
            6 => self.volume.get(),
            7 => self.attack.get(),
            8 => self.decay.get(),
            9 => self.sustain.get(),
            10 => self.release.get(),
            11 => self.filter_type.get(),
            12 => self.filter_cutoff.get(),
            13 => self.filter_resonance.get(),            
            14 => self.filter_attack.get(),
            15 => self.filter_decay.get(),
            16 => self.filter_sustain.get(),
            17 => self.filter_release.get(),
            18 => self.pitch_lfo_depth.get(),
            19 => self.pitch_lfo_rate.get(),
            _ => 0.0
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.oscillator_one_type.set(value),
            1 => self.oscillator_one_pulsewidth.set(value),
            2 => self.oscillator_one_octave.set(value),
            3 => self.oscillator_two_type.set(value),
            4 => self.oscillator_two_pulsewidth.set(value),
            5 => self.oscillator_two_octave.set(value),
            6 => self.volume.set(value),
            7 => self.attack.set(value),
            8 => self.decay.set(value),
            9 => self.sustain.set(value),
            10 => self.release.set(value),
            11 => self.filter_type.set(value),
            12 => self.filter_cutoff.set(value),
            13 => self.filter_resonance.set(value),
            14 => self.filter_attack.set(value),
            15 => self.filter_decay.set(value),
            16 => self.filter_sustain.set(value),
            17 => self.filter_release.set(value),
            18 => self.pitch_lfo_depth.set(value),
            19 => self.pitch_lfo_rate.set(value),
            _ => ()
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {                        
            7 => "s",
            8 => "s",            
            10 => "s",            
            12 => "Hz",
            14 => "s",
            15 => "s",
            17 => "s",
            18 => "%",
            19 => "Hz",
            _ => ""
        }.to_string()
    }
}

impl Plugin for Synth {
    fn new(_host: HostCallback) -> Self {        
        Synth {
            oscillators_one: vec![oscillator::Oscillator::default(), oscillator::Oscillator::default(), oscillator::Oscillator::default(), oscillator::Oscillator::default()],            
            oscillators_two: vec![oscillator::Oscillator::default(), oscillator::Oscillator::default(), oscillator::Oscillator::default(), oscillator::Oscillator::default()],            
            pitch_lfo: oscillator::LFO::default(),
            params: Arc::new(SynthParameters::default()),        
            last_played_osc_index: 0,
            filters: vec![filter::Filter::default(), filter::Filter::default(), filter::Filter::default(), filter::Filter::default()],
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
            parameters: 20,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // i would very much like to NOT have to calculate these each buffer - need a way to call from the parameters set_parameter...        
        for i in 0..4 {
            self.oscillators_one[i].set_params(match self.params.oscillator_one_type.get() {
                t if t < 0.25 => OscillatorType::Saw,
                t if t < 0.5 => OscillatorType::Pulse,
                t if t < 0.75 => OscillatorType::Triangle,
                _ => OscillatorType::Sin
            }, self.params.oscillator_one_pulsewidth.get(), self.params.oscillator_one_octave.get());

            self.oscillators_one[i].envelope.set_params(
                self.params.attack.get().powi(2) * 10.0, 
                self.params.decay.get().powi(2) * 10.0, 
                self.params.sustain.get(), 
                self.params.release.get().powi(2) * 10.0
            );

            self.oscillators_two[i].set_params(match self.params.oscillator_two_type.get() {
                t if t < 0.25 => OscillatorType::Saw,
                t if t < 0.5 => OscillatorType::Pulse,
                t if t < 0.75 => OscillatorType::Triangle,
                _ => OscillatorType::Sin
            }, self.params.oscillator_two_pulsewidth.get(), self.params.oscillator_two_octave.get());

            self.oscillators_two[i].envelope.set_params(
                self.params.attack.get().powi(2) * 10.0, 
                self.params.decay.get().powi(2) * 10.0, 
                self.params.sustain.get(), 
                self.params.release.get().powi(2) * 10.0
            );

            self.filters[i].set_params(
                self.params.filter_cutoff.get().powi(3), 
                self.params.filter_resonance.get(), 
                self.params.filter_type.get(), 
                self.params.filter_attack.get().powi(2) * 10.0, 
                self.params.filter_decay.get().powi(2) * 10.0, 
                self.params.filter_sustain.get(), 
                self.params.filter_release.get().powi(2) * 10.0
            );   
        }        

        self.pitch_lfo.set_params(self.params.pitch_lfo_depth.get().powi(2), (self.params.pitch_lfo_rate.get() * 19.9) + 0.1);
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();                

        for sample in 0..samples {
            let pitch_lfo_amt = self.pitch_lfo.process();
            let mut sample_value = 0.0;
            for i in 0..4 {
                sample_value += self.filters[i].process(self.oscillators_one[i].process_with_pitch_mod(pitch_lfo_amt) * self.params.volume.get());
                sample_value += self.filters[i].process(self.oscillators_two[i].process_with_pitch_mod(pitch_lfo_amt) * self.params.volume.get());
            }
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
        for i in 0..4 {
            self.oscillators_one[i].set_sample_rate(self.sample_rate);
            self.oscillators_two[i].set_sample_rate(self.sample_rate);
            self.filters[i].set_sample_rate(self.sample_rate);
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
        self.oscillators_one[self.last_played_osc_index].note_on(note, vel); 
        self.oscillators_two[self.last_played_osc_index].note_on(note, vel);
        self.filters[self.last_played_osc_index].start_note();
        self.last_played_osc_index = (self.last_played_osc_index + 1) % 4;              
    }

    fn note_off(&mut self, note: u8) {     
        for i in 0..4 {
            if self.oscillators_one[i].get_current_note() == note {
                self.oscillators_one[i].note_off();
                self.filters[i].end_note();
            }
            if self.oscillators_two[i].get_current_note() == note {
                self.oscillators_two[i].note_off();
            }
        }
    }
}

plugin_main!(Synth);