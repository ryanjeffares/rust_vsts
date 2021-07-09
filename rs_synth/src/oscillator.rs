use crate::adsr;

const TWO_PI: f32 = std::f32::consts::PI * 2.0;

fn mtof(note: u8) -> f32 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f32 = 440.0;
    ((f32::from(note as i8 - A4_PITCH)) / 12.0).exp2() * A4_FREQ
}

pub struct Oscillator {
    frequency: f32,
    velocity: f32,
    note: u8,
    phase: f32,
    output: f32,
    pulsewidth: f32,
    sample_rate: f32,
    octave_mod: f32,
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

pub enum OscillatorType {
    Saw, Pulse, Sin, Triangle
}

impl Default for Oscillator {
    fn default() -> Self {
        Oscillator {
            frequency: 261.63,
            velocity: 0.0,
            note: 60,
            phase: 0.0,
            output: 0.0,
            pulsewidth: 0.5,
            sample_rate: 44100.0,
            octave_mod: 1.0,
            osc_type: OscillatorType::Saw,
            envelope: adsr::ADSR::default()
        }
    }
}

impl Oscillator {
    pub fn note_on(&mut self, note: u8, vel: u8) {
        self.velocity = vel as f32 / 127.0;
        self.frequency = mtof(note);
        self.note = note;
        self.envelope.start_note();
    }

    pub fn note_off(&mut self) {
        self.envelope.end_note();
    }

    pub fn get_current_note(&self) -> u8 {
        self.note
    }

    pub fn set_params(&mut self, osc_type: OscillatorType, pw: f32, octave: f32) {
        self.osc_type = osc_type;
        self.pulsewidth = pw;
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
        let freq = self.frequency * self.octave_mod;
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
        let freq = (self.frequency * pitch_mod) * self.octave_mod;        
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