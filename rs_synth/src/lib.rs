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
*   Poly dual osc synth with switchable waveforms and resonant lowpass/highpass/bandpass filter
*   and all the usual good shit you'd expect from a synth
*   TODO:
*   Pitch envelope
*   Cross Modulation
*   Filter/freq lfo
*/

const VOICES: usize = 8;

#[derive(Default)]
struct Synth {
    oscillators_one: Vec<oscillator::Oscillator>,    
    oscillators_two: Vec<oscillator::Oscillator>,    
    pitch_lfo: oscillator::LFO,
    params: Arc<SynthParameters>,
    last_played_osc_index: usize,
    current_num_voices: usize,
    monophonic: bool,
    filters: Vec<filter::Filter>,
    active_notes: Vec<u8>,
    active_velocities: Vec<u8>,
    sample_rate: f32
}

struct SynthParameters {
    oscillator_one_type: AtomicFloat,
    oscillator_one_pulsewidth: AtomicFloat,
    oscillator_one_octave: AtomicFloat,
    oscillator_one_semitone: AtomicFloat,
    oscillator_one_fine: AtomicFloat,
    oscillator_one_volume: AtomicFloat,
    oscillator_two_type: AtomicFloat,
    oscillator_two_pulsewidth: AtomicFloat,
    oscillator_two_octave: AtomicFloat,
    oscillator_two_semitone: AtomicFloat,
    oscillator_two_fine: AtomicFloat,
    oscillator_two_volume: AtomicFloat,    
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
    pitch_lfo_rate: AtomicFloat,
    num_voices: AtomicFloat,
    portamento: AtomicFloat,
}

impl Default for SynthParameters {
    fn default() -> SynthParameters {
        SynthParameters {
            oscillator_one_type: AtomicFloat::new(0.0),
            oscillator_one_pulsewidth: AtomicFloat::new(0.5),
            oscillator_one_octave: AtomicFloat::new(0.5),
            oscillator_one_semitone: AtomicFloat::new(0.5),
            oscillator_one_fine: AtomicFloat::new(0.5),
            oscillator_one_volume: AtomicFloat::new(0.05),
            oscillator_two_type: AtomicFloat::new(0.0),
            oscillator_two_pulsewidth: AtomicFloat::new(0.5),
            oscillator_two_octave: AtomicFloat::new(0.5),
            oscillator_two_semitone: AtomicFloat::new(0.5),
            oscillator_two_fine: AtomicFloat::new(0.5),
            oscillator_two_volume: AtomicFloat::new(0.05),            
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
            pitch_lfo_rate: AtomicFloat::new(0.25),
            num_voices: AtomicFloat::new(1.0),
            portamento: AtomicFloat::new(0.0) 
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
            3 => format!("{}{}", if self.oscillator_one_semitone.get() > 0.5 { "+" } else { "" }, ((self.oscillator_one_semitone.get() * 48.0) - 24.0) as i8),
            4 => format!("{}{}", if self.oscillator_one_fine.get() > 0.5 { "+" } else { "" }, ((self.oscillator_one_fine.get() * 200.0) - 100.0) as i8),
            5 => format!("{:.2}", self.oscillator_one_volume.get()),
            6 => match self.oscillator_two_type.get() {
                t if t < 0.25 => "Saw",
                t if t < 0.5 => "Pulse",
                t if t < 0.75 => "Tri",
                _ => "Sine"
            }.to_string(),
            7 => format!("{:.2}", self.oscillator_two_pulsewidth.get()),
            8 => match self.oscillator_two_octave.get() {
                o if o < 0.2 => "-2",
                o if o < 0.4 => "-1",
                o if o < 0.6 => "0",
                o if o < 0.8 => "+1",
                _ => "+2"
            }.to_string(),
            9 => format!("{}{}", if self.oscillator_two_semitone.get() > 0.5 { "+" } else { "" }, ((self.oscillator_two_semitone.get() * 48.0) - 24.0) as i8),
            10 => format!("{}{}", if self.oscillator_two_fine.get() > 0.5 { "+" } else { "" }, ((self.oscillator_two_fine.get() * 200.0) - 100.0) as i8),            
            11 => format!("{}", self.oscillator_two_volume.get()),
            12 => format!("{:.2}", self.attack.get().powi(2) * 10.0),
            13 => format!("{:.2}", self.decay.get().powi(2) * 10.0),
            14 => format!("{:.2}", self.sustain.get()),
            15 => format!("{:.2}", self.release.get().powi(2) * 10.0),            
            16 => if self.filter_type.get() < 0.33 { "Lowpass" } else if self.filter_type.get() < 0.66 { "Bandpass" } else { "Highpass" }.to_string(),
            17 => format!("{:.2}", (self.filter_cutoff.get().powi(3) * 19980.0) + 20.0),
            18 => format!("{:.2}", (self.filter_resonance.get() * 9.9) + 0.1),
            19 => format!("{:.2}", self.filter_attack.get().powi(2) * 10.0),
            20 => format!("{:.2}", self.filter_decay.get().powi(2) * 10.0),
            21 => format!("{:.2}", self.filter_sustain.get()),
            22 => format!("{:.2}", self.filter_release.get().powi(2) * 10.0),
            23 => format!("{:.2}", self.pitch_lfo_depth.get().powi(2) * 100.0),
            24 => format!("{:.2}", (self.pitch_lfo_rate.get() * 19.9) + 0.1),
            25 => format!("{}", ((self.num_voices.get() * 7.0) + 1.0) as u8),
            26 => format!("{:.2}", (self.portamento.get().powi(4) * 9.999) + 0.001),
            _ => "".to_string()
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Osc 1 Type",
            1 => "Osc 1 Pulsewidth",
            2 => "Osc 1 Octave",
            3 => "Osc 1 Semitone",
            4 => "Osc 1 Fine",
            5 => "Osc 1 Volume",
            6 => "Osc 2 Type",
            7 => "Osc 2 Pulsewidth",
            8 => "Osc 2 Octave",
            9 => "Osc 2 Semitone",
            10 => "Osc 2 Fine",
            11 => "Osc 2 Volume",
            12 => "Envelope Attack",
            13 => "Envelope Decay",
            14 => "Envelope Sustain",
            15 => "Envelope Release",            
            16 => "Filter Type",
            17 => "Filter Cutoff",
            18 => "Filter Resonance",
            19 => "Filter Attack",
            20 => "Filter Decay",
            21 => "Filter Sustain",
            22 => "Filter Release",
            23 => "Pitch LFO Depth",
            24 => "Pitch LFO Rate",
            25 => "Voices",
            26 => "Portamento Time",
            _ => ""
        }.to_string()
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.oscillator_one_type.get(),
            1 => self.oscillator_one_pulsewidth.get(),
            2 => self.oscillator_one_octave.get(),
            3 => self.oscillator_one_semitone.get(),
            4 => self.oscillator_one_fine.get(),
            5 => self.oscillator_one_volume.get(),
            6 => self.oscillator_two_type.get(),
            7 => self.oscillator_two_pulsewidth.get(),
            8 => self.oscillator_two_octave.get(),
            9 => self.oscillator_two_semitone.get(),
            10 => self.oscillator_two_fine.get(),
            11 => self.oscillator_two_volume.get(),            
            12 => self.attack.get(),
            13 => self.decay.get(),
            14 => self.sustain.get(),
            15 => self.release.get(),
            16 => self.filter_type.get(),
            17 => self.filter_cutoff.get(),
            18 => self.filter_resonance.get(),            
            19 => self.filter_attack.get(),
            20 => self.filter_decay.get(),
            21 => self.filter_sustain.get(),
            22 => self.filter_release.get(),
            23 => self.pitch_lfo_depth.get(),
            24 => self.pitch_lfo_rate.get(),
            25 => self.num_voices.get(),
            26 => self.portamento.get(),
            _ => 0.0
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.oscillator_one_type.set(value),
            1 => self.oscillator_one_pulsewidth.set(value),
            2 => self.oscillator_one_octave.set(value),
            3 => self.oscillator_one_semitone.set(value),
            4 => self.oscillator_one_fine.set(value),
            5 => self.oscillator_one_volume.set(value),
            6 => self.oscillator_two_type.set(value),
            7 => self.oscillator_two_pulsewidth.set(value),
            8 => self.oscillator_two_octave.set(value),
            9 => self.oscillator_two_semitone.set(value),
            10 => self.oscillator_two_fine.set(value),
            11 => self.oscillator_two_volume.set(value),            
            12 => self.attack.set(value),
            13 => self.decay.set(value),
            14 => self.sustain.set(value),
            15 => self.release.set(value),
            16 => self.filter_type.set(value),
            17 => self.filter_cutoff.set(value),
            18 => self.filter_resonance.set(value),
            19 => self.filter_attack.set(value),
            20 => self.filter_decay.set(value),
            21 => self.filter_sustain.set(value),
            22 => self.filter_release.set(value),
            23 => self.pitch_lfo_depth.set(value),
            24 => self.pitch_lfo_rate.set(value),
            25 => self.num_voices.set(value),
            26 => self.portamento.set(value),
            _ => ()
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            4 => "ct",
            10 => "ct",                      
            12 => "s",
            13 => "s",            
            15 => "s",            
            17 => "Hz",
            19 => "s",
            20 => "s",
            22 => "s",
            23 => "%",
            24 => "Hz",
            26 => "s",
            _ => ""
        }.to_string()
    }
}

impl Plugin for Synth {
    fn new(_host: HostCallback) -> Self {        
        Synth {
            oscillators_one: vec![oscillator::Oscillator::default(); VOICES],            
            oscillators_two: vec![oscillator::Oscillator::default(); VOICES],            
            pitch_lfo: oscillator::LFO::default(),
            params: Arc::new(SynthParameters::default()),        
            last_played_osc_index: 0,
            current_num_voices: 8,
            monophonic: false,
            filters: vec![filter::Filter::default(); VOICES],
            active_notes: vec![],
            active_velocities: vec![],
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
            parameters: 27,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // check to see if the amount of voices has gone down - cancel any notes that we need to if it has
        let voices = ((self.params.num_voices.get() * 7.0) + 1.0) as usize;
        if self.current_num_voices > voices {
            for i in voices..VOICES {
                self.oscillators_one[i].note_off();
                self.oscillators_two[i].note_off();
                self.filters[i].end_note();
            }
            // make sure the next note we play is on the first oscillator in the vectors
            if voices == 1 {
                self.last_played_osc_index = 0;
            }
        }
        self.current_num_voices = voices;
        self.monophonic = self.current_num_voices == 1;
        // i would very much like to NOT have to calculate these each buffer - need a way to call from the parameters set_parameter...        
        for i in 0..VOICES {
            self.oscillators_one[i].set_params(
                    match self.params.oscillator_one_type.get() {
                    t if t < 0.25 => OscillatorType::Saw,
                    t if t < 0.5 => OscillatorType::Pulse,
                    t if t < 0.75 => OscillatorType::Triangle,
                    _ => OscillatorType::Sin
                }, 
                self.params.oscillator_one_pulsewidth.get(), 
                self.params.oscillator_one_octave.get(),
                (self.params.portamento.get().powi(4) * 9.999) + 0.001,
                (self.params.oscillator_one_semitone.get() * 48.0) as i8 - 24,
                (self.params.oscillator_one_fine.get() * 200.0) - 100.0
            );

            self.oscillators_one[i].envelope.set_params(
                self.params.attack.get().powi(2) * 10.0, 
                self.params.decay.get().powi(2) * 10.0, 
                self.params.sustain.get(), 
                self.params.release.get().powi(2) * 10.0
            );

            self.oscillators_two[i].set_params(
                match self.params.oscillator_two_type.get() {
                    t if t < 0.25 => OscillatorType::Saw,
                    t if t < 0.5 => OscillatorType::Pulse,
                    t if t < 0.75 => OscillatorType::Triangle,
                    _ => OscillatorType::Sin
                }, 
                self.params.oscillator_two_pulsewidth.get(), 
                self.params.oscillator_two_octave.get(),
                (self.params.portamento.get().powi(4) * 9.999) + 0.001,
                (self.params.oscillator_two_semitone.get() * 48.0) as i8 - 24,
                (self.params.oscillator_two_fine.get() * 200.0) - 100.0
            );

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
            for i in 0..VOICES {
                let samp = 
                    (self.oscillators_one[i].process_with_pitch_mod(pitch_lfo_amt) * self.params.oscillator_one_volume.get()) 
                    + (self.oscillators_two[i].process_with_pitch_mod(pitch_lfo_amt) * self.params.oscillator_two_volume.get());
                sample_value += self.filters[i].process(samp);
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
        for i in 0..VOICES {
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
        self.oscillators_one[self.last_played_osc_index].note_on(note, vel, self.monophonic); 
        self.oscillators_two[self.last_played_osc_index].note_on(note, vel, self.monophonic);        
        self.filters[self.last_played_osc_index].start_note();
        self.last_played_osc_index = (self.last_played_osc_index + 1) % self.current_num_voices;
        if !self.active_notes.contains(&note) {
            self.active_notes.push(note);
            self.active_velocities.push(vel);          
        }
    }

    fn note_off(&mut self, note: u8) {     
        for i in 0..VOICES {
            if self.oscillators_one[i].get_current_note() == note {
                self.oscillators_one[i].note_off();
                self.filters[i].end_note();
            }
            if self.oscillators_two[i].get_current_note() == note {
                self.oscillators_two[i].note_off();
            }
        }
        if self.active_notes.contains(&note) {
            let idx = self.active_notes.iter().position(|x| *x == note).unwrap();
            self.active_notes.remove(idx);
            self.active_velocities.remove(idx);
        }
        if self.monophonic && self.active_notes.len() > 0 {
            self.note_on(*self.active_notes.last().unwrap(), *self.active_velocities.last().unwrap());
        }
    }
}

plugin_main!(Synth);