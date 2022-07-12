fn update() {
    
}

fn main() {
    kinc::KincBuilder::new("Kinc + Rust", 500, 500).update_callback(update).build().start();
}
