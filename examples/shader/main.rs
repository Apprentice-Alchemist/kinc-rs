use kinc::{ClearMode, Kinc};

fn update(kinc: &Kinc) {
    let g4 = kinc.default_window().g4();

    g4.begin();

    g4.clear(ClearMode::COLOR, 0xFF0FFFFF, 1.0, 1);

    g4.end();

    kinc::g4::swap_buffers().unwrap();
}

fn main() {
    let (kinc, _) = kinc::KincBuilder::new("Kinc + Rust", 500, 500)
        .update_callback(update)
        .build();

    kinc.start();
}
