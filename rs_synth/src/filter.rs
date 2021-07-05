#[derive(Clone, Copy)]
pub struct Filter {
    cutoff: f32,
    resonance: f32,
    sample_rate: f32,
    speed: f32,
    pos: f32,
    filter_type: FilterType
}

#[derive(Clone, Copy)]
pub enum FilterType {
    Lowpass, Highpass
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            cutoff: 20000.0,
            resonance: 1.0,
            sample_rate: 44100.0,
            speed: 0.0,
            pos: 0.0,
            filter_type: FilterType::Lowpass
        }
    }
}

impl Filter {
    pub fn set_type(&mut self, new_type: FilterType) {
        self.filter_type = new_type;
    }

    pub fn set_params(&mut self, cutoff: f32, res: f32) {
        self.cutoff = cutoff;
        self.resonance = res;
    }

    pub fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        match self.filter_type {
            FilterType::Lowpass => self.lowpass(input),
            FilterType::Highpass => self.highpass(input)
        }
    }

    fn lowpass(&mut self, input: f32) -> f32 {
        let z = ((std::f32::consts::PI * 2.0) * self.cutoff / self.sample_rate).cos();
        let c = 2.0 - (2.0 * z);        
        // (sqrt(2.0) * sqrt(-pow((z - 1.0), 3.0)) + resonance * (z - 1)) / (resonance * (z - 1))
        let r = (std::f32::consts::SQRT_2 * (-(z - 1.0).powi(3)).sqrt() + self.resonance * (z - 1.0)) / (self.resonance * (z - 1.0));
        self.speed += (input - self.pos) * c;
        self.pos += self.speed;
        self.speed *= r;
        self.pos
    }

    fn highpass(&mut self, input: f32) -> f32 {
        input - self.lowpass(input)
    }
}
