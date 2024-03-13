fn main() {
    println!("cargo:rerun-if-env-changed=CONDA_PREFIX");
    let prefix = std::env::var("CONDA_PREFIX").expect("build in conda env");
    let conda = std::path::Path::new(&prefix).join("lib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", conda.display());
}
