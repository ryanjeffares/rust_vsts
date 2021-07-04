use crate::adsr;

fn mtof(note: u8) -> f32 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f32 = 440.0;
    ((f32::from(note as i8 - A4_PITCH)) / 12.0).exp2() * A4_FREQ
}

#[derive(Copy, Clone)]
pub struct Oscillator {
    frequency: f32,
    note: u8,
    phase: f32,
    output: f32,
    sample_rate: f32,
    osc_type: OscillatorType,
    pub envelope: adsr::ADSR
}

#[derive(Copy, Clone)]
pub enum OscillatorType {
    Saw, Square
}

impl Default for Oscillator {
    fn default() -> Self {
        Oscillator {
            frequency: 261.63,
            note: 60,
            phase: 0.0,
            output: 0.0,
            sample_rate: 44100.0,
            osc_type: OscillatorType::Saw,
            envelope: adsr::ADSR::default()
        }
    }
}

impl Oscillator {
    pub fn new() -> Self {
        Oscillator {
            frequency: 261.63,
            note: 60,
            phase: 0.0,
            output: 0.0,
            sample_rate: 44100.0,
            osc_type: OscillatorType::Saw,
            envelope: adsr::ADSR::default()
        }
    }

    pub fn note_on(&mut self, note: u8) {
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

    pub fn set_type(&mut self, osc_type: OscillatorType) {
        self.osc_type = osc_type;
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.envelope.set_sample_rate(self.sample_rate);
    }

    // must call every sample
    pub fn process(&mut self) -> f32 {
        self.envelope.process();
        match self.osc_type {
            OscillatorType::Saw => {
                self.output = self.phase;
                if self.phase >= 1.0 {
                    self.phase -= 2.0;
                }
                self.phase += (1.0 / (self.sample_rate / self.frequency)) * 2.0;
                self.output * self.envelope.get_output()
            }
            OscillatorType::Square => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase += 1.0 / (self.sample_rate / self.frequency);
                if self.phase > 0.5 { 1.0 * self.envelope.get_output() } else { -1.0 * self.envelope.get_output() }
            }            
        }
    }
}