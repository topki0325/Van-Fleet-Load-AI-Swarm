use eframe::egui;

// Task view component (initial extraction).
#[derive(Debug, Default)]
pub struct TaskComponent;

impl TaskComponent {
    pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut crate::app::VgaGuiApp) {
        ui.heading(app.tr("任务", "Task"));
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label(app.tr("语言", "Language"));
            ui.text_edit_singleline(&mut app.task_language);
            ui.label(app.tr("目标", "Target"));
            ui.text_edit_singleline(&mut app.task_target);
        });

        ui.label(app.tr("上下文", "Context"));
        ui.text_edit_multiline(&mut app.task_context);

        ui.horizontal_wrapped(|ui| {
            if ui.button(app.tr("执行", "Execute")).clicked() {
                app.execute_task();
            }
            if ui.button(app.tr("提交", "Submit")).clicked() {
                app.submit_task();
            }
        });
    }
}
