fn main() {
    if let Ok("release") = std::env::var("PROFILE").as_deref() {
        println!("cargo:rustc-cfg=release");
    }
}
