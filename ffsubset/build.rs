fn main() {
    let rdkit = "/home/brent/omsf/clone/rdkit/build/lib";
    let include = "/home/brent/Projects/rdkit-sys/include";

    println!("cargo:rustc-link-arg=-Wl,-rpath,{include}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{rdkit}");
}
