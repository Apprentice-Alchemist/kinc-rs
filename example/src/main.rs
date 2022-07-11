fn main() {
    println!("Hello, world!");
    kinc::init(
        "Shader",
        500,
        500,
        None,
        None,
    );

    kinc::start();
}
