use rand::Rng;
use std::os::raw::c_char;
use std::ffi::{CString};

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

const TAU : f32 = 3.141 * 2.;

const BASE_RATE : f32 = TAU / (120.);
const MIN_RATE : f32 = BASE_RATE * 0.50;
const MAX_RATE : f32 = BASE_RATE * 2.25;

impl World {
    fn new(osc_count : usize) -> Self {
        let mut oscs = Vec::with_capacity(osc_count);

        let levels = 4;

        for i in 0..osc_count {
            let amp = ((i % levels) + 1) as f32 / (4.*(levels as f32));
            oscs.push(Oscillator {
                pos : i as f32 / (osc_count as f32),
                //rate : (3.141 * 2.) / ((1. + amp) * 120.),
                rate : BASE_RATE,
                amp,
                t : (i * 100) as f32,
            });
        }

        World {
            oscs,
        }
    }

    fn tick(&mut self) {
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

struct Oscillator
{
    t : f32,
    pos : f32,
    rate : f32,
    amp : f32,
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

        println!("Sample before {}", self.sample());

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

        println!("Moving delta={}", delta);
        println!("Sample after {}", self.sample());
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
pub extern "C" fn add_world(generators : f64) -> f64 {
    unsafe {
        let id = GLOBAL_STATE.as_ref().unwrap().worlds.len();
        GLOBAL_STATE.as_mut().unwrap().worlds.push(World::new(generators as usize));
        id as f64
    }
}

#[no_mangle]
pub extern "C" fn sample(world_id: f64, pos : f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        world.sample(pos as f32) as f64
    }
}

#[no_mangle]
pub extern "C" fn sample_osc(world_id: f64, osc : f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        let osc = &world.oscs[osc as usize];
        osc.sample() as f64
    }
}

#[no_mangle]
pub extern "C" fn get_amp(world_id: f64, osc : f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        let osc = &world.oscs[osc as usize];
        osc.amp as f64
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

#[no_mangle]
pub extern "C" fn slam(world_id: f64, pos : f64, force : f64) -> f64 {
    unsafe {
        let world = &mut GLOBAL_STATE.as_mut().unwrap().worlds[world_id as usize];
        world.slam(force as f32, pos as f32);

        0.0
    }
}

#[no_mangle]
pub extern "C" fn tick() -> f64 {
    unsafe {
        let state = &mut GLOBAL_STATE.as_mut().unwrap();
        for world in &mut state.worlds
        {
            world.tick();
        }

        0.0
    }
}

#[no_mangle]
pub extern "C" fn osc_count(world_id: f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        world.oscs.len() as f64
    }
}

static mut LAST_DEBUG_STRING : Option<CString> = None;

#[no_mangle]
pub extern "C" fn osc_debug(world_id: f64, osc_id : f64) -> *const c_char {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        let osc = &world.oscs[osc_id as usize];

        let c_str = CString::new(format!("pos: {}, rate: {} amp: {}", osc.pos, osc.rate, osc.amp)).unwrap();
        LAST_DEBUG_STRING = Some(c_str);
        LAST_DEBUG_STRING.as_ref().unwrap().as_ptr()
    }
}