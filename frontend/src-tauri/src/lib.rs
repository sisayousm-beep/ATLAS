//! Tauri desktop shell for Project Atlas (UI roadmap Phase 1).
//!
//! Hosts the `atlas-sim` engine: a background thread steps the world in real time
//! (one in-game day per tick, paced by the game speed) and emits a `world` event
//! with a fresh snapshot after each step. The client reads the snapshot and drives
//! the loop through the `get_world` / `get_status` / `set_paused` / `set_speed`
//! commands.

use atlas_sim::{Simulation, WorldSnapshot};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{Emitter, State};

/// The running simulation plus its loop controls, shared between the ticking
/// thread and the command handlers.
struct SimState {
    sim: Mutex<Simulation>,
    paused: AtomicBool,
    /// Game speed in in-game days per real second.
    speed: AtomicU32,
}

/// Loop state the client mirrors for its controls.
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SimStatus {
    paused: bool,
    speed: u32,
}

/// Current world snapshot (also pushed via the `world` event each tick).
#[tauri::command]
fn get_world(state: State<Arc<SimState>>) -> WorldSnapshot {
    state.sim.lock().unwrap().snapshot()
}

/// Current pause/speed, for the client to sync its controls on start-up.
#[tauri::command]
fn get_status(state: State<Arc<SimState>>) -> SimStatus {
    SimStatus {
        paused: state.paused.load(Ordering::Relaxed),
        speed: state.speed.load(Ordering::Relaxed),
    }
}

#[tauri::command]
fn set_paused(state: State<Arc<SimState>>, paused: bool) {
    state.paused.store(paused, Ordering::Relaxed);
}

#[tauri::command]
fn set_speed(state: State<Arc<SimState>>, speed: u32) {
    state.speed.store(speed.clamp(1, 60), Ordering::Relaxed);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(SimState {
        sim: Mutex::new(Simulation::new()),
        paused: AtomicBool::new(false),
        speed: AtomicU32::new(1),
    });

    tauri::Builder::default()
        .manage(state.clone())
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Drive the world on a background thread, pushing a snapshot each tick.
            let handle = app.handle().clone();
            let sim_state = state.clone();
            thread::spawn(move || {
                // Paint the client immediately with the starting world.
                let initial = sim_state.sim.lock().unwrap().snapshot();
                let _ = handle.emit("world", initial);

                loop {
                    if sim_state.paused.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    let snapshot = {
                        let mut sim = sim_state.sim.lock().unwrap();
                        sim.step();
                        sim.snapshot()
                    };
                    let _ = handle.emit("world", snapshot);

                    let speed = sim_state.speed.load(Ordering::Relaxed).max(1);
                    thread::sleep(Duration::from_millis(1000 / speed as u64));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_world,
            get_status,
            set_paused,
            set_speed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
