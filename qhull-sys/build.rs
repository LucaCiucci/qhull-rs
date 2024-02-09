use std::{env, fs::read_dir, path::PathBuf};

const QHULL_SRC_DIR: &str = "qhull/src/libqhull_r";

fn main() {
    println!("cargo:rerun-if-changed=src/error_handling.h");
    println!("cargo:rerun-if-changed=src/error_handling.c");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_triple = env::var("TARGET").unwrap();

    let all_headers = std::env::var("CARGO_FEATURE_ALL_HEADERS").is_ok();
    let include_programs = std::env::var("CARGO_FEATURE_INCLUDE_PROGRAMS").is_ok();

    let mut sources = vec![];
    let mut headers = vec![];

    for entry in read_dir(QHULL_SRC_DIR).unwrap() {
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

    let mut builder = cc::Build::new();
    builder.files(sources.iter().map(|s| format!("{}/{}", QHULL_SRC_DIR, s)));
    builder.file("src/error_handling.c");
    builder.include(QHULL_SRC_DIR);
    builder.include("qhull/src");

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

    let mut bindings_builder = bindgen::Builder::default()
        .header(wrapper.to_str().unwrap())
        .header("src/error_handling.h")
        .use_core() // no_std
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args([
            "-Iqhull/src/libqhull_r".to_string(),
            "-Iqhull/src".to_string(),
            "-target".to_string(),
            target_triple,
        ]);

    if include_programs {
        let programs = [
            ("qconvex", "qconvex_r"),
            ("qdelaunay", "qdelaun_r"),
            ("qhalf", "qhalf_r"),
            ("qhull", "unix_r"),
            ("qvoronoi", "qvoronoi_r"),
            ("rbox", "rbox_r"), // https://github.com/rust-lang/cc-rs/issues/809 $env:VSLANG=1033
        ];

        let mut header = String::new();
        let mut programs_changed = false;

        for (program, main_file) in &programs {
            let program_path = format!("qhull/src/{program}/{main_file}.c");
            let main_function_name = format!("qhull_sys__{}_main", program);
            let program_source = std::fs::read_to_string(&program_path)
                .unwrap()
                .replace("int main(", &format!("int {}(", main_function_name));
            // write the modified source to a file in the OUT_DIR
            let program_source_path = out_path.join(format!("{}.c", program));
            let current_content = std::fs::read_to_string(&program_source_path).unwrap_or_default();
            // avoids recompiling if the file hasn't changed
            if current_content != program_source {
                std::fs::write(&program_source_path, program_source).unwrap();
                programs_changed = true;
            }
            // add the file to the build
            builder.file(program_source_path.to_str().unwrap());
            // add the main function to the wrapper
            header.push_str(&format!("int {}(int argc, char* argv[]);\n", main_function_name));
        }

        // write the header to a file in the OUT_DIR
        let header_path = out_path.join("qhull_programs.h");
        if programs_changed {
            std::fs::write(&header_path, header).unwrap();
        }
        bindings_builder = bindings_builder.header(header_path.to_str().unwrap());
    }


    builder.compile("qhull_r");

    let bindings = bindings_builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
