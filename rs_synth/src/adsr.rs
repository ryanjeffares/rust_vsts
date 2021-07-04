pub struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    sample_rate: f64,
    note_on: bool,
    attack_samples : i32,
    decay_samples : i32,
    release_samples: i32,
    sample_counter: i32,
    should_release: bool
}

pub enum ADSRParams {
    Attack, Decay, Sustain, Release
}

impl Default for ADSR {
    fn default() -> ADSR {
        ADSR {
            attack: 0.2,
            decay: 0.2,
            sustain: 0.8,
            release: 0.5,
            sample_rate: 44100.0,
            note_on: false,
            attack_samples: 8820,            
            decay_samples: 8820,
            release_samples: 22050,
            sample_counter: 0,
            should_release: false
        }
    }
}

impl ADSR {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32, sample_rate: f64) -> Self {
        let att_samples = (attack * (sample_rate as f32)) as i32;
        let dec_samples = (decay * (sample_rate as f32)) as i32;
        let rel_samples = (release * (sample_rate as f32)) as i32;
        ADSR {
            attack,
            decay, 
            sustain,
            release, 
            sample_rate,
            note_on: false,
            attack_samples: att_samples,
            decay_samples: dec_samples,
            release_samples: rel_samples,
            sample_counter: 0,
            should_release: false
        }
    }

    pub fn start_note(&mut self) {
        self.note_on = true;
    }

    pub fn end_note(&mut self) {
        self.should_release = true;    
        self.note_on = false;    
    }

    pub fn set_param(&mut self, param: ADSRParams, value: f32) {
        match param {
            ADSRParams::Attack => self.attack = value,            
            ADSRParams::Decay => self.decay = value,            
            ADSRParams::Sustain => self.sustain = value,            
            ADSRParams::Release => self.release = value               
        }
        self.attack_samples = (self.attack * (self.sample_rate as f32)) as i32;
        self.decay_samples = (self.decay * (self.sample_rate as f32)) as i32;
        self.release_samples = (self.release * (self.sample_rate as f32)) as i32;
    }

    pub fn set_sample_rate(&mut self, sr: f64) {
        self.sample_rate = sr;
    }

    // should be called on every sample
    pub fn get_value(&mut self) -> f32 {
        if !self.note_on {    
            if self.should_release {
                let num_samps = self.sample_counter - (self.attack_samples + self.decay_samples);
                self.sample_counter += 1;
                1.0 - ((num_samps / self.release_samples) as f32)                        
            }
            else {                
                0.0
            }
        }
        else {
            self.sample_counter += 1;
            if self.sample_counter < self.attack_samples {
                (self.sample_counter / self.attack_samples) as f32
            }
            else if self.sample_counter < self.decay_samples {
                ((self.sample_counter - self.attack_samples) / self.decay_samples) as f32
            }
            else {
                0.0
            }
        }
    }
}