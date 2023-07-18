fn main() {
    let output = "app.gresource";
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/app.gresource.xml",
        output,
    );
    println!("cargo:rerun-if-changed=resources/*");
    println!("cargo:rerun-if-changed=build.rs");
}
