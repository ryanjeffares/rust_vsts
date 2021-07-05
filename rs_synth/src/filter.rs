pub struct Filter {    
    sample_rate: f32,
    y: [f32; 3],
    s1: f32,
    s2: f32,
    filter_state: FilterState,
    filter_type: FilterType
}

struct FilterState {
    g: f32,
    r2: f32,
    h: f32
}

impl Default for FilterState {
    fn default() -> Self {
        let g = (std::f32::consts::PI * 200.0 / 44100.0).tan();
        let r2 = std::f32::consts::SQRT_2;
        let h = 1.0 / (1.0 + r2 * g + g * g);
        FilterState {
            g: g,
            r2: r2,
            h: h
        }
    }
}

impl FilterState {
    pub fn set_params(&mut self, sample_rate: f32, freq: f32, resonance: f32) {
        self.g = (std::f32::consts::PI * freq / sample_rate).tan();
        self.r2 = 1.0 / resonance;
        self.h = 1.0 / (1.0 + self.r2 * self.g + self.g * self.g);
    }
}

pub enum FilterType {
    Lowpass, Bandpass, Highpass
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            sample_rate: 44100.0,
            y: [0.0, 0.0, 0.0],
            s1: 0.0,
            s2: 0.0,
            filter_state: FilterState::default(),
            filter_type: FilterType::Lowpass
        }
    }
}

impl Filter {
    pub fn set_params(&mut self, cutoff: f32, res: f32, new_type: f32, sample_rate: f32) {
        self.filter_state.set_params(sample_rate, (cutoff * 19980.0) + 20.0, (res * 9.9) + 0.1);
        self.filter_type = match new_type {
            t if t < 0.33 => FilterType::Lowpass,
            t if t < 0.66 => FilterType::Bandpass,
            _ => FilterType::Highpass
        }
    }

    pub fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        self.y[2] = (input - self.s1 * self.filter_state.r2 - self.s1 * self.filter_state.g - self.s2) * self.filter_state.h;

        self.y[1] = self.y[2] * self.filter_state.g + self.s1;

        self.s1 = self.y[2] * self.filter_state.g + self.y[1];

        self.y[0] = self.y[1] * self.filter_state.g + self.s2;

        self.s2 = self.y[1] * self.filter_state.g + self.y[0];

        match self.filter_type {
            FilterType::Lowpass => self.y[0],
            FilterType::Bandpass => self.y[1],
            FilterType::Highpass => self.y[2]
        }
    }
}
