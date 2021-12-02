mod app;

pub use egui_miniquad as emq;
pub use miniquad as mq;

fn main() {
    let conf = mq::conf::Conf {
        window_title: "Potential".to_string(),
        high_dpi: true,
        window_resizable: false,
        ..Default::default()
    };
    
    mq::start(conf, |mut ctx| {
        mq::UserData::owning(app::Potential::new(&mut ctx), ctx)
    })
}
