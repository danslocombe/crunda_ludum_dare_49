use rand::Rng;

struct GlobalState
{
    worlds: Vec<World>,
}

impl Default for GlobalState
{
    fn default() -> Self {
        Self {
            worlds: vec![],
        }
    }
}

struct World
{
    oscs: Vec<Oscillator>,
}

impl World {
    fn new(osc_count : usize) -> Self {
        let mut oscs = Vec::with_capacity(osc_count);

        let levels = 4;

        for i in 0..osc_count {
            let amp = (i % levels) as f32 / levels as f32;
            oscs.push(Oscillator {
                pos : i as f32 / (osc_count as f32),
                //rate : (3.141 * 2.) / ((1. + amp) * 120.),
                rate : (3.141 * 2.) / (120.),
                amp,
                t_off : (i * 100) as u32,
            });
        }

        World {
            oscs,
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
    pub fn sample(&self, t : u32, pos : f32) -> f32 {
        // For now sample all using some simple decay func

        let mut res = 0.;
        for osc in &self.oscs {
            let dist = min_dist(osc.pos, pos);
            // Hack
            let tt = (t as f32).ln() + 1200.0;
            //let k = t as f32 / 800.;
            let k = tt / 800.;
            let weighting = distance_weight_log(dist, k);

            if (weighting.is_finite() && !weighting.is_nan() && weighting > 0.0)
            {
                res += distance_weight_log(dist, k) * osc.sample(t);
            }
        }

        res
    }

    pub fn add_weight(&mut self, delta_weight : f32, pos : f32) {
        for osc in &mut self.oscs {
            let dist = min_dist(osc.pos, pos);
            let delta = delta_weight * distance_weight_log(dist, 4.);
            //osc.amp = (osc.amp + delta).clamp(0.0, 1.0);
            osc.rate = (osc.rate + delta).clamp(0.0001, 1.0);
        }
    }
}

struct Oscillator
{
    t_off : u32,
    pos : f32,
    rate : f32,
    amp : f32,
}

impl Oscillator {
    // Gives result in [-1, 1]
    pub fn sample(&self, t : u32) -> f32 {
        self.amp * ((t + self.t_off) as f32 * self.rate).sin()
    }
}

static mut GLOBAL_STATE : Option<GlobalState> = None;

#[no_mangle]
pub extern "C" fn reset() -> f64 {
    unsafe {
        GLOBAL_STATE = Some(GlobalState::default());
    }
    0.0
}

#[no_mangle]
pub extern "C" fn add_world() -> f64 {
    unsafe {
        let id = GLOBAL_STATE.as_ref().unwrap().worlds.len();
        GLOBAL_STATE.as_mut().unwrap().worlds.push(World::new(32));
        id as f64
    }
}

#[no_mangle]
pub extern "C" fn sample(world_id: f64, pos : f64, t : f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        world.sample(t as u32, pos as f32) as f64
    }
}

#[no_mangle]
pub extern "C" fn add_weight(world_id: f64, pos : f64, mag : f64) -> f64 {
    unsafe {
        let world = &mut GLOBAL_STATE.as_mut().unwrap().worlds[world_id as usize];
        world.add_weight(mag as f32, pos as f32);

        0.0
    }
}