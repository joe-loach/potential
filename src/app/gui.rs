use super::App;

impl App {
    pub(crate) fn ui(&mut self) {
        let ctx = self.emq.egui_ctx();

        egui::Window::new("Test").show(ctx, |_ui| {});
    }
}
