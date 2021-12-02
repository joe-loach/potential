mod app;

pub use egui_miniquad as emq;
pub use miniquad as mq;

fn main() {
    let conf = mq::conf::Conf {
        window_title: "Potential".to_string(),
        high_dpi: true,
        ..Default::default()
    };

    mq::start(conf, |mut ctx| {
        mq::UserData::owning(app::App::new(&mut ctx), ctx)
    })
}
