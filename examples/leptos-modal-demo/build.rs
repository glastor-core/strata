//! Zero-Node.js CSS bundling build script for leptos-modal-demo.

fn main() {
    println!("cargo:rerun-if-changed=src");

    glastor_css_binder::BuildBinder::new()
        .scan_dir("src")
        .output_css("dist/style.min.css")
        .compile()
        .expect("Failed to compile zero-Node CSS bundle");
}
