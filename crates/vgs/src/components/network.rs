use eframe::egui;

#[derive(Debug, Default)]
pub struct NetworkComponent;

impl NetworkComponent {
    pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut crate::VgaGuiApp) {
        ui.heading(app.tr("网络", "Network"));
        ui.separator();

        if ui.button(app.tr("发现节点", "Discover Peers")).clicked() {
            app.discover_peers();
        }

        egui::ScrollArea::vertical()
            .id_source("network_peers_scroll")
            .max_height(260.0)
            .show(ui, |ui| {
                ui.monospace(&app.peers_json);
            });
    }
}
