// Alias `vas_core` to the name used in the GUI code.
extern crate vas_core as vangriten_ai_swarm;

mod components;
mod app_types;   // UiLang, ProviderFilter, ActiveView
mod app;         // VgaGuiApp struct + constructor + helpers
mod app_actions; // VgaGuiApp backend action methods
mod app_ui;      // render_api_manager_window + eframe::App impl

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("vas")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "vas",
        native_options,
        Box::new(|cc| {
            if let Err(err) = egui_chinese_font::setup_chinese_fonts(&cc.egui_ctx) {
                eprintln!("Failed to load Chinese fonts: {err}");
            }
            Box::new(app::VgaGuiApp::new())
        }),
    )
}

