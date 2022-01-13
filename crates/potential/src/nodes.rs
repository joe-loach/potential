use archie::egui::{self, Pos2, Rect, Response, Ui, Vec2};

pub struct NodePanel;

impl NodePanel {
    pub fn ui<'a>(nodes: impl IntoIterator<Item = Node<'a>>, ui: &mut Ui) {
        for (id, node) in nodes.into_iter().enumerate() {
            node.window(id, ui);
        }
    }
}

pub struct Node<'a> {
    title: String,
    header: Option<Box<dyn FnOnce(&mut Ui) -> Response + 'a>>,
    pos: Pos2,
    body: Option<Box<dyn FnOnce(&mut Ui) -> Response + 'a>>,
}

impl<'a> Node<'a> {
    const SIZE: Vec2 = Vec2::splat(100.0);

    pub fn new(title: impl Into<String>, pos: impl Into<Pos2>) -> Self {
        Self {
            title: title.into(),
            header: None,
            pos: pos.into(),
            body: None,
        }
    }

    pub fn with_header(mut self, header: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        self.header.replace(Box::new(header));
        self
    }

    pub fn with_body(mut self, body: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        self.body.replace(Box::new(body));
        self
    }
}

impl<'a> Node<'a> {
    pub fn window(self, id: usize, ui: &mut Ui) {
        let rect = Rect::from_center_size(self.pos, Self::SIZE);
        let bounds = ui.min_rect();
        let translation = ui.min_rect().min - Pos2::ZERO;

        let frame = egui::Frame::window(ui.style()).shadow({
            let mut shadow = ui.style().visuals.window_shadow;
            shadow.extrusion = 1.0;
            shadow
        });
        egui::Window::new(self.title.clone())
            .id(egui::Id::new(id))
            .frame(frame)
            .collapsible(false)
            .vscroll(false)
            .resizable(false)
            .default_size(rect.size())
            .default_pos(rect.min + translation)
            .title_bar(false)
            .drag_bounds(bounds)
            .show(ui.ctx(), |ui| {
                self.ui(ui);
            });
    }

    pub fn ui(self, ui: &mut Ui) {
        let Self { header, body, .. } = self;

        if let Some(header) = header {
            ui.allocate_ui(ui.available_size(), header);
        }
        if let Some(body) = body {
            ui.separator();
            ui.allocate_ui(ui.available_size(), body);
        }
    }
}
