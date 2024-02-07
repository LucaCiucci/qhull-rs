use std::{env, fs::read_dir, path::{Path, PathBuf}};


const QHULL_SRC_DIR: &str = "qhull/src/libqhull_r";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_triple = env::var("TARGET").unwrap();

    let all_headers = std::env::var("CARGO_FEATURE_ALL_HEADERS").is_ok();

    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(Path::new(QHULL_SRC_DIR));

    let mut sources = vec![];
    let mut headers = vec![];

    for entry in read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let file = path.file_name().unwrap().to_str().unwrap().to_string();
            if file.ends_with(".c") {
                sources.push(file);
            } else if file.ends_with(".h") {
                headers.push(file);
            }
        }
    }

    cc::Build::new()
        .files(sources.iter().map(|s| format!("{}/{}", QHULL_SRC_DIR, s)))
        .include(QHULL_SRC_DIR)
        .compile("qhull_r");

    let wrapper = if all_headers {
        // create a wrapper file
        let mut wrapper = String::new();
        for include in headers {
            wrapper.push_str(&format!("#include <{}>\n", include));
        }
        let wrapper_path = out_path.join("qhull_wrapper.h");
        std::fs::write(&wrapper_path, wrapper).unwrap();
        wrapper_path
    } else {
        println!("cargo:rerun-if-changed=wrapper.h");
        PathBuf::from("wrapper.h")
    };

    let bindings = bindgen::Builder::default()
        .header(wrapper.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args([
            "-Iqhull/src/libqhull_r".to_string(),
            "-target".to_string(), target_triple,
        ])
        .generate()
        .expect("Unable to generate bindings");

    let out_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}