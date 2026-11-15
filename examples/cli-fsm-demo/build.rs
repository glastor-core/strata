//! Zero-Node.js CSS bundling build script for cli-fsm-demo.

fn main() {
    glastor_css_binder::BuildBinder::new()
        .scan_dir("src")
        .output_css("dist/style.min.css")
        .compile()
        .expect("CSS bundling compilation failed");
}
