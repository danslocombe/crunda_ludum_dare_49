pub struct World
{
    pub oscs: Vec<Oscillator>,
}

const TAU : f32 = 3.141 * 2.;

const BASE_RATE : f32 = TAU / (120.);
const MIN_RATE : f32 = BASE_RATE * 0.50;
const MAX_RATE : f32 = BASE_RATE * 2.25;

impl World {
    pub fn new(seed : usize, osc_count : usize) -> Self {
        let mut oscs = Vec::with_capacity(osc_count);

        let levels = 4;

        for i in 0..osc_count {
            let amp = (((i + seed) % levels) + 1) as f32 / (4.*(levels as f32));
            oscs.push(Oscillator {
                pos : i as f32 / (osc_count as f32),
                //rate : (3.141 * 2.) / ((1. + amp) * 120.),
                rate : BASE_RATE,
                amp,
                t : (seed * 1235 + i * 100) as f32,
            });
        }

        World {
            oscs,
        }
    }

    pub fn tick(&mut self) {
        for osc in &mut self.oscs {
            osc.tick();
        }
    }
}

fn angle_diff(x : f32, y : f32) -> f32 {
    let diff = y - x;

    if diff > 0.5 {
        diff - 1.0
    }
    else if diff < -0.5{
        diff  + 1.0
    }
    else {
        diff
    }
}

fn min_dist(x : f32, y : f32) -> f32 {
    angle_diff(x, y).abs()
}

fn distance_weight(dist : f32, k : f32) -> f32 {
    //const K : f32 = 4.;
    1. / (1. + k*dist)
}

fn distance_weight_log(dist: f32, k: f32) -> f32 {
    1. + (1. - k * dist).ln()
}

impl World {
    // Gives result in [-1, 1]
    pub fn sample(&self, pos : f32) -> f32 {
        // For now sample all using some simple decay func

        let mut res = 0.;
        for osc in &self.oscs {
            let dist = min_dist(osc.pos, pos);
            // Hack
            //let tt = (osc.t as f32).ln() + 1200.0;
            //let k = t as f32 / 800.;
            //let k = tt / 800.;
            //let weighting = distance_weight_log(dist, k);
            let k = 50.;
            let weighting = 1.0 / (1.0 + k*dist);

            if (weighting.is_finite() && !weighting.is_nan() && weighting > 0.0)
            {
                //res += distance_weight_log(dist, k) * osc.sample();
                res += weighting * osc.sample();
            }
        }

        res
    }

    pub fn add_weight(&mut self, delta_weight : f32, pos : f32) {
        for osc in &mut self.oscs {
            let dist = min_dist(osc.pos, pos);
            //let dist_weighting = distance_weight_log(dist, 4.);
            let k = 100.;
            let distance_weighting = (k*dist).sin()/(k*dist);
            //if (dist_weighting > 0.)
            {
                let delta = delta_weight * distance_weighting;
                osc.update_amp(delta);
                osc.update_rate(1. * delta);
            }
        }
    }

    pub fn slam(&mut self, force : f32, pos : f32) {
        for osc in &mut self.oscs {
            let dist = min_dist(osc.pos, pos);
            if (dist < 0.125)
            {
                //let k = 500.;
                //let weighting = 1.0 / (1.0 + k*dist);
                let weighting = 1.0 - 20.0*dist;
                if (weighting > 0.0) {
                    osc.slam(force * weighting);
                }
            }
        }
    }
}

pub struct Oscillator
{
    t : f32,
    pub pos : f32,
    pub rate : f32,
    pub amp : f32,
}

impl Oscillator {
    // Gives result in [-1, 1]
    pub fn sample(&self) -> f32 {
        self.amp * (self.t).sin()
    }

    pub fn tick(&mut self) {
        self.t += self.rate;
    }

    pub fn update_amp(&mut self, delta : f32) {
        self.amp = (self.amp + delta).clamp(0.0, 1.0);
    }

    pub fn update_rate(&mut self, delta : f32) {
        self.rate = (self.rate + 1.* delta).clamp(MIN_RATE, BASE_RATE);
    }

    pub fn slam(&mut self, force : f32) {

        self.update_amp(force);
        self.update_rate(-force);

        let move_val = force * 1.0;

        // Find next trough
        // Move towards

        //println!("Sample before {}", self.sample());

        let a = (self.t / TAU).floor();
        let b = self.t - TAU*a;

        let b_target = if (b < 0.25 * TAU) {
            // Go backwards
            -0.25 * TAU
        }
        else if (b > 0.75 * TAU) {
            // Go backwards
            0.75 * TAU
        }
        else {
            // Go forwards
            0.75 * TAU
        };

        let mut delta = (b_target - b);

        if (delta.abs() > move_val)
        {
            delta = delta.signum() * move_val;
        }

        self.t += delta;

        ////println!("Moving delta={}", delta);
        ////println!("Sample after {}", self.sample());
    }
}
