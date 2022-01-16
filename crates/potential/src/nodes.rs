use archie::egui::{self, pos2, Pos2, Rect, Response, Ui, Vec2};

pub struct NodePanel;

impl NodePanel {
    pub fn ui<'a>(nodes: impl IntoIterator<Item = Node<'a>>, ui: &mut Ui) {
        let mut windows: Vec<Rect> = Vec::new();
        let available_rect = ui.min_rect();
        let position = |existing: &[Rect]| {
            let spacing = 4.0;
            let left = available_rect.left() + spacing;
            let top = available_rect.top() + spacing;

            if existing.is_empty() {
                return pos2(left, top);
            }

            // Separate existing rectangles into columns:
            let mut column_bbs = vec![existing[0]];

            for &rect in existing.iter() {
                let current_column_bb = column_bbs.last_mut().unwrap();
                if rect.left() < current_column_bb.right() {
                    // same column
                    *current_column_bb = current_column_bb.union(rect);
                } else {
                    // new column
                    column_bbs.push(rect);
                }
            }

            {
                // Look for large spaces between columns (empty columns):
                let mut x = left;
                for col_bb in &column_bbs {
                    let available = col_bb.left() - x;
                    if available >= 100.0 {
                        return pos2(x, top);
                    }
                    x = col_bb.right() + spacing;
                }
            }

            // Find first column with some available space at the bottom of it:
            for col_bb in &column_bbs {
                if col_bb.bottom() < available_rect.center().y {
                    return pos2(col_bb.left(), col_bb.bottom() + spacing);
                }
            }

            // Maybe we can fit a new column?
            let rightmost = column_bbs.last().unwrap().right();
            if rightmost + 100.0 < available_rect.right() {
                return pos2(rightmost + spacing, top);
            }

            // Ok, just put us in the column with the most space at the bottom:
            let mut best_pos = pos2(left, column_bbs[0].bottom() + spacing);
            for col_bb in &column_bbs {
                let col_pos = pos2(col_bb.left(), col_bb.bottom() + spacing);
                if col_pos.y < best_pos.y {
                    best_pos = col_pos;
                }
            }
            best_pos
        };

        for (i, node) in nodes.into_iter().enumerate() {
            let id = ui.id().with(i);
            let pos = position(&windows);
            let response = node.window(id, pos, ui);
            windows.push(response.rect);
        }
    }
}

pub struct Node<'a> {
    header: Option<Box<dyn FnOnce(&mut Ui) -> Response + 'a>>,
    body: Option<Box<dyn FnOnce(&mut Ui) -> Response + 'a>>,
}

impl<'a> Node<'a> {
    const SIZE: Vec2 = Vec2::new(80.0, 100.0);

    pub fn new() -> Self {
        Self {
            header: None,
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
    pub(self) fn window(self, id: egui::Id, pos: Pos2, ui: &mut Ui) -> egui::Response {
        let rect = Rect::from_min_size(pos, Self::SIZE);
        let bounds = ui.min_rect();

        let frame = egui::Frame::window(ui.style()).shadow({
            let mut shadow = ui.style().visuals.window_shadow;
            shadow.extrusion = 1.0;
            shadow
        });
        egui::Window::new("")
            .id(id)
            .frame(frame)
            .collapsible(false)
            .vscroll(false)
            .resizable(false)
            .default_size(rect.size())
            .default_pos(rect.min)
            .title_bar(false)
            .drag_bounds(bounds)
            .show(ui.ctx(), |ui| {
                self.ui(ui);
            })
            .unwrap() // always open
            .response
    }

    fn ui(self, ui: &mut Ui) {
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
