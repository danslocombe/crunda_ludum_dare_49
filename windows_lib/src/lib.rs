use world_generators_core::*;

use std::os::raw::c_char;
use std::ffi::{CString};

static mut GLOBAL_STATE : Option<GlobalState> = None;

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
        GLOBAL_STATE.as_mut().unwrap().worlds.push(World::new(id, generators as usize));
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