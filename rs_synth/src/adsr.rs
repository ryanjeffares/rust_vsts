// 100% knicked from https://github.com/BelaPlatform/Bela/tree/master/libraries/ADSR

fn calculate_coefficient(rate: f32, ratio: f32) -> f32 {
    let log = -((1.0 + ratio) / ratio).log(std::f32::consts::E) / rate;        
    log.exp()
}

#[derive(Clone)]
pub struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,    
    attack_coeff: f32,
    decay_coeff: f32,
    release_coeff: f32,
    attack_target_ratio: f32,
    dr_target_ratio: f32,    
    attack_base: f32,
    decay_base: f32,
    release_base: f32,
    state: ADSRState,
    sample_rate: f32,
    output: f32
}

#[derive(PartialEq, Clone)]
enum ADSRState {
    Idle, Attack, Decay, Sustain, Release
}

impl Default for ADSR {
    fn default() -> ADSR {
        let attack_coeff = calculate_coefficient(0.2, 0.3);
        let decay_coeff = calculate_coefficient(0.2, 0.0001);
        let release_coeff = calculate_coefficient(0.5, 0.0001);
        ADSR {
            attack: 0.2,
            decay: 0.1,
            sustain: 0.8,
            release: 0.5,                        
            attack_coeff: attack_coeff,
            decay_coeff: decay_coeff,
            release_coeff: release_coeff,
            attack_target_ratio: 0.3,
            dr_target_ratio: 0.0001,
            attack_base: (1.0 + 0.3) * (1.0 - attack_coeff),
            decay_base: (1.0 - 0.0001) * (1.0 - decay_coeff),
            release_base: -0.0001 * (1.0 - release_coeff),
            state: ADSRState::Idle,
            sample_rate: 44100.0,
            output: 0.0
        }
    }
}

impl ADSR {

    pub fn start_note(&mut self) {
        self.state = ADSRState::Attack;
    }

    pub fn end_note(&mut self) {         
        if self.state != ADSRState::Idle {
            self.state = ADSRState::Release;
        }
    }

    pub fn reset(&mut self) {
        self.state = ADSRState::Idle;
        self.output = 0.0;
    }

    pub fn get_output(&self) -> f32 {
        self.output
    }

    pub fn set_params(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.calculate_attack(attack * self.sample_rate);
        self.calculate_decay(decay * self.sample_rate);           
        self.calculate_sustain(sustain);     
        self.calculate_release(release * self.sample_rate);        
    }   

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    pub fn process(&mut self) {
        match self.state {
            ADSRState::Idle => (),
            ADSRState::Attack => {
                self.output = self.attack_base + self.output * self.attack_coeff;
                if self.output >= 1.0 {
                    self.output = 1.0;
                    self.state = ADSRState::Decay;
                }
            },
            ADSRState::Decay => {
                self.output = self.decay_base + self.output * self.decay_coeff;
                if self.output <= self.sustain {
                    self.output = self.sustain;
                    self.state = ADSRState::Sustain;
                }
            },
            ADSRState::Sustain => (),
            ADSRState::Release => {
                self.output = self.release_base + self.output * self.release_coeff;
                if self.output <= 0.0 {
                    self.output = 0.0;
                    self.state = ADSRState::Idle;
                }
            }
        }        
    }
    
    fn calculate_attack(&mut self, attack: f32) {
        self.attack = attack;
        self.attack_coeff = calculate_coefficient(self.attack, self.attack_target_ratio);
        self.attack_base = (1.0 + self.attack_target_ratio) * (1.0 - self.attack_coeff);
    }

    fn calculate_decay(&mut self, decay: f32) {
        self.decay = decay;
        self.decay_coeff = calculate_coefficient(self.decay, self.dr_target_ratio);
        self.decay_base = (self.sustain - self.dr_target_ratio) * (1.0 - self.decay_coeff);
    }

    fn calculate_sustain(&mut self, sustain: f32) {
        self.sustain = sustain;
        self.decay_base = (self.sustain - self.dr_target_ratio) * (1.0 - self.decay_coeff);
    }

    fn calculate_release(&mut self, release: f32) {
        self.release = release;
        self.release_coeff = calculate_coefficient(self.release, self.dr_target_ratio);
        self.release_base = -self.dr_target_ratio * (1.0 - self.release_coeff);
    }    

    fn set_attack_target_ratio(&mut self, mut ratio: f32) {
        if ratio < 0.000000001 {
            ratio = 0.000000001;
        }
        self.attack_target_ratio = ratio;
        self.attack_base = (1.0 + self.attack_target_ratio) * (1.0 - self.attack_coeff);
    }

    fn set_dr_target_ratio(&mut self, mut ratio: f32) {
        if ratio < 0.000000001 {
            ratio = 0.000000001;
        }
        self.dr_target_ratio = ratio;
        self.decay_base = (self.sustain - self.dr_target_ratio) * (1.0 - self.decay_coeff);
        self.release_base = -self.dr_target_ratio * (1.0 - self.release_coeff);
    }
}