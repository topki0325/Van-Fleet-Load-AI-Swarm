// Keep the existing GUI implementation without rewriting all paths:
// alias `vas_core` to the name used in the GUI code.
extern crate vas_core as vangriten_ai_swarm;

mod components;

include!("../../../src/bin/vga_gui.rs");
