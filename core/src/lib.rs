// A World is made up of a collection of oscillators equally spaced in a circle.
pub struct World
{
    pub oscs: Vec<Oscillator>,
}

const TAU : f32 = 3.141 * 2.;

const BASE_RATE : f32 = TAU / 120.;
const MIN_RATE : f32 = BASE_RATE * 0.50;

impl World {
    pub fn new(seed : usize, osc_count : usize) -> Self {
        let mut oscs = Vec::with_capacity(osc_count);

        let levels = 4;

        for i in 0..osc_count {
            // In the compo game we don't really vary the frequency of the osciallators and setup the amplitudes
            // to a regular pattern.
            let amp = (((i + seed) % levels) + 1) as f32 / (4.*(levels as f32));
            oscs.push(Oscillator {
                pos : i as f32 / (osc_count as f32),
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

// The smallest difference between two angles represented as numbers in [0-1)
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
    1. / (1. + k*dist)
}

fn distance_weight_log(dist: f32, k: f32) -> f32 {
    1. + (1. - k * dist).ln()
}

impl World {
    // Sample the surface level of a world at a given "world position" or angle represented as a number in [0,1)
    // The actual radius is then rendered as r_pos = r0 + r_vary * sample(pos)
    //
    // Gives result in [-1, 1]
    pub fn sample(&self, pos : f32) -> f32 {

        // For now sample all oscilators using some simple weighting func
        let mut res = 0.;
        for osc in &self.oscs {
            let dist = min_dist(osc.pos, pos);
            let k = 50.;
            let weighting = 1.0 / (1.0 + k*dist);

            if weighting > 0.0
            {
                res += weighting * osc.sample();
            }
        }

        res
    }

    // Add weight to the osciallators near a point
    // Used for debugging but not in final entry 
    pub fn add_weight(&mut self, delta_weight : f32, pos : f32) {
        for osc in &mut self.oscs {
            let dist = min_dist(osc.pos, pos);
            let k = 100.;
            let distance_weighting = (k*dist).sin()/(k*dist);
            {
                let delta = delta_weight * distance_weighting;
                osc.update_amp(delta);
                osc.update_rate(1. * delta);
            }
        }
    }

    // Simulate an object slamming into the surface
    // Basically we change the value of t of all oscillators close to the impact position.
    pub fn slam(&mut self, force : f32, pos : f32) {
        for osc in &mut self.oscs {
            let dist = min_dist(osc.pos, pos);
            if dist < 0.125
            {
                let weighting = 1.0 - 20.0*dist;
                if weighting > 0.0 {
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

        // If you imagine the oscillator tracing a sine wave, we want to
        // "move" the current time of the oscillator towards a trough (-1)
        //
        //                    _ _
        //  |               /     \
        //  |\            /         \
        //  |  |         |           |
        //  -----------------------------------------------
        //  |  |         |
        //  |    \ _ _ /
        //  |
        //          |
        //          target
        //

        // We start by finding the current cycle number we are on and isolating focusing just on that
        let a = (self.t / TAU).floor();
        let b = self.t - TAU*a;

        // b is the local position in the current cycle and b_target will be the local minimum
        // We decide b_target by looking at where we are in the current cycle
        // We either want target_0 or target_1
        //
        //               _ _
        //          | /     \
        //          |/       \
        //          |         |
        //          ---------------------------------------------
        //   |      |          |       |
        //    \_ _ /|           \ _ _ /
        //      |   |
        //      |   0              |
        //      |                  target_1
        //      target_0

        let b_target = if b < 0.25 * TAU {
            -0.25 * TAU
        }
        else {
            0.75 * TAU
        };

        let mut delta = b_target - b;

        // If the difference between b and b_target is greater than the max move value determined by the force
        // we cap it.
        if delta.abs() > move_val
        {
            delta = delta.signum() * move_val;
        }

        self.t += delta;
    }
}
