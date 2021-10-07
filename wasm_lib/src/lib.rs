use wasm_bindgen::prelude::*;
use world_generators_core::*;
/*
    TODO really the two libs should be in one project where we have some macro for exporting for
    windows or wasm
    But this is for a game jam..
*/

/*
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, World!");
}

#[wasm_bindgen]
pub fn reset() {
    alert("RESET FROM WASM RUIST!");
}
*/

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

#[wasm_bindgen]
pub fn reset() {
    unsafe {
        GLOBAL_STATE = Some(GlobalState::default());
    }
}

#[wasm_bindgen]
pub fn add_world(generators : f64) -> f64 {
    unsafe {
        let id = GLOBAL_STATE.as_ref().unwrap().worlds.len();
        GLOBAL_STATE.as_mut().unwrap().worlds.push(World::new(id, generators as usize));
        id as f64
    }
}

#[wasm_bindgen]
pub fn sample(world_id: f64, pos : f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        world.sample(pos as f32) as f64
    }
}

#[wasm_bindgen]
pub fn sample_osc(world_id: f64, osc : f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        let osc = &world.oscs[osc as usize];
        osc.sample() as f64
    }
}

#[wasm_bindgen]
pub fn get_amp(world_id: f64, osc : f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        let osc = &world.oscs[osc as usize];
        osc.amp as f64
    }
}

#[wasm_bindgen]
pub fn add_weight(world_id: f64, pos : f64, mag : f64) {
    unsafe {
        let world = &mut GLOBAL_STATE.as_mut().unwrap().worlds[world_id as usize];
        world.add_weight(mag as f32, pos as f32);
    }
}

#[wasm_bindgen]
pub fn slam(world_id: f64, pos : f64, force : f64) {
    unsafe {
        let world = &mut GLOBAL_STATE.as_mut().unwrap().worlds[world_id as usize];
        world.slam(force as f32, pos as f32);
    }
}

#[wasm_bindgen]
pub fn tick() {
    unsafe {
        let state = &mut GLOBAL_STATE.as_mut().unwrap();
        for world in &mut state.worlds
        {
            world.tick();
        }
    }
}

#[wasm_bindgen]
pub fn osc_count(world_id: f64) -> f64 {
    unsafe {
        let world = &GLOBAL_STATE.as_ref().unwrap().worlds[world_id as usize];
        world.oscs.len() as f64
    }
}

/*
UNUSED in prod
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
*/