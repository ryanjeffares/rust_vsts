use crate::adsr;

const TWO_PI: f32 = std::f32::consts::PI * 2.0;
const A4_PITCH: u8 = 69;
const A4_FREQ: f32 = 440.0;

fn mtof(note: u8) -> f32 {    
    (f32::from(note - A4_PITCH) / 12.0).exp2() * A4_FREQ
}

#[derive(Clone)]
pub struct Oscillator {
    frequency: f32,
    previous_frequency: f32,
    port_time: f32,
    monophonic: bool,
    sample_counter: u32,
    velocity: f32,
    note: u8,
    phase: f32,
    output: f32,
    pulsewidth: f32,
    sample_rate: f32,
    octave_mod: f32,
    semitone_mod: i8,
    fine_mod: f32,
    osc_type: OscillatorType,    
    pub envelope: adsr::ADSR
}

pub struct LFO {
    frequency: f32,
    phase: f32,
    output: f32,
    depth: f32,     
    sample_rate: f32
}

#[derive(Clone)]
pub enum OscillatorType {
    Saw, Pulse, Sin, Triangle
}

impl Default for Oscillator {
    fn default() -> Self {
        Oscillator {
            frequency: 261.63,
            previous_frequency: 261.63,
            port_time: 0.0,
            monophonic: false,
            sample_counter: 0,
            velocity: 0.0,
            note: 60,
            phase: 0.0,
            output: 0.0,
            pulsewidth: 0.5,
            sample_rate: 44100.0,
            octave_mod: 1.0,
            semitone_mod: 0,
            fine_mod: 0.0,
            osc_type: OscillatorType::Saw,
            envelope: adsr::ADSR::default()
        }
    }
}

impl Oscillator {
    pub fn note_on(&mut self, note: u8, vel: u8, mono: bool) {
        self.velocity = vel as f32 / 127.0;
        self.monophonic = mono;
        if mono {
            self.previous_frequency = self.frequency;
            self.frequency = mtof(note);
            self.sample_counter = 0;
        }
        else {
            self.frequency = mtof(note);
            self.previous_frequency = self.frequency;            
        }
        self.note = note;
        self.envelope.start_note();
    }

    pub fn note_off(&mut self) {
        self.envelope.end_note();
    }

    pub fn get_current_note(&self) -> u8 {
        self.note
    }

    pub fn set_params(&mut self, osc_type: OscillatorType, pw: f32, octave: f32, port_time: f32, semitone: i8, fine: f32) {
        self.osc_type = osc_type;
        self.pulsewidth = pw;
        self.port_time = port_time;
        self.semitone_mod = semitone;
        self.fine_mod = fine;
        // octave will be 0 - 1 float, need to translate that to -2 to 2 multiplier...        
        self.octave_mod = match octave {
            o if o < 0.2 => 0.25,
            o if o < 0.4 => 0.5,
            o if o < 0.6 => 1.0,
            o if o < 0.8 => 2.0,
            _ => 3.0
        }        
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.envelope.set_sample_rate(self.sample_rate);        
    }

    // must call every sample
    pub fn process(&mut self) -> f32 {
        self.envelope.process();
        let mut freq;
        if self.monophonic {
            let elapsed_port_time = self.sample_counter as f32 / self.sample_rate;
            if elapsed_port_time < self.port_time {      
                self.sample_counter += 1;
                freq = self.previous_frequency + 
                    ((self.frequency - self.previous_frequency) * (elapsed_port_time / self.port_time));
            }
            else {
                freq = self.frequency;
            }
        }  
        else {
            freq = self.frequency;
        }        
        freq *= 2.0f32.powf(((self.semitone_mod as f32 * 100.0) + self.fine_mod) / 1200.0);
        freq *= self.octave_mod;        
        match self.osc_type {
            OscillatorType::Saw => {
                self.output = self.phase;
                if self.phase >= 1.0 {
                    self.phase -= 2.0;
                }
                self.phase += (1.0 / (self.sample_rate / freq)) * 2.0;                                
                self.output * self.envelope.get_output() * self.velocity
            }
            OscillatorType::Pulse => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase += 1.0 / (self.sample_rate / freq);
                if self.phase < self.pulsewidth { 
                    1.0 * self.envelope.get_output() * self.velocity
                } else { 
                    -1.0 * self.envelope.get_output() * self.velocity
                }
            }
            OscillatorType::Sin => {
                self.output = (self.phase * TWO_PI).sin();
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase += 1.0 / (self.sample_rate / freq);
                self.output * self.envelope.get_output() * self.velocity
            }
            OscillatorType::Triangle => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase += 1.0 / (self.sample_rate / freq);
                if self.phase <= 0.5 {
                    self.output = (self.phase - 0.25) * 4.0;
                }
                else {
                    self.output = ((1.0 - self.phase) - 0.25) * 4.0;
                }
                self.output * self.envelope.get_output() * self.velocity
            } 
        }
    }

    pub fn process_with_pitch_mod(&mut self, pitch_mod: f32) -> f32 {
        self.envelope.process();
        let mut freq;        
        if self.monophonic {
            let elapsed_port_time = self.sample_counter as f32 / self.sample_rate;
            if elapsed_port_time < self.port_time {      
                self.sample_counter += 1;
                freq = self.previous_frequency + 
                    ((self.frequency - self.previous_frequency) * (elapsed_port_time / self.port_time));
                freq += freq * pitch_mod;
            }
            else {
                freq = self.frequency + (self.frequency * pitch_mod);
            }  
        }
        else {
            freq = self.frequency + (self.frequency * pitch_mod);
        }    
        freq *= 2.0f32.powf(((self.semitone_mod as f32 * 100.0) + self.fine_mod) / 1200.0);
        freq *= self.octave_mod;
        match self.osc_type {
            OscillatorType::Saw => {
                self.output = self.phase;
                if self.phase >= 1.0 {
                    self.phase -= 2.0;
                }
                self.phase += (1.0 / (self.sample_rate / freq)) * 2.0;
                self.output * self.envelope.get_output() * self.velocity
            }
            OscillatorType::Pulse => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase += 1.0 / (self.sample_rate / freq);
                if self.phase < self.pulsewidth { 
                    1.0 * self.envelope.get_output() * self.velocity
                } else { 
                    -1.0 * self.envelope.get_output() * self.velocity
                }
            }
            OscillatorType::Sin => {
                self.output = (self.phase * TWO_PI).sin();
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase += 1.0 / (self.sample_rate / freq);
                self.output * self.envelope.get_output() * self.velocity
            }
            OscillatorType::Triangle => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase += 1.0 / (self.sample_rate / freq);
                if self.phase <= 0.5 {
                    self.output = (self.phase - 0.25) * 4.0;
                }
                else {
                    self.output = ((1.0 - self.phase) - 0.25) * 4.0;
                }
                self.output * self.envelope.get_output() * self.velocity
            }        
        }
    }
}

impl Default for LFO {
    fn default() -> Self {
        LFO {
            frequency: 5.0,
            phase: 0.0,
            output: 0.0,
            depth: 0.0,
            sample_rate: 44100.0
        }
    }
}

impl LFO {
    pub fn set_params(&mut self, depth: f32, frequency: f32) {
        self.frequency = frequency;
        self.depth = depth;
    }

    pub fn process(&mut self) -> f32 {
        self.output = (self.phase * TWO_PI).sin();
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        self.phase += 1.0 / (self.sample_rate / self.frequency);
        self.output * self.depth
    }
}