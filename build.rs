use std::{
    env,
    error::Error,
    fs::read_dir,
    path::Path,
    process::{exit, Command},
    str,
};

const LLVM_MAJOR_VERSION: usize = if cfg!(feature = "llvm16-0") {
    16
} else if cfg!(feature = "llvm17-0") {
    17
} else if cfg!(feature = "llvm18-0") {
    18
} else {
    19
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let version = llvm_config("--version")?;

    if !version.starts_with(&format!("{LLVM_MAJOR_VERSION}.")) {
        return Err(format!(
            "failed to find correct version ({LLVM_MAJOR_VERSION}.x.x) of llvm-config (found {version})",
        )
        .into());
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=cc");
    println!("cargo:rustc-link-search={}", llvm_config("--libdir")?);

    build_c_library()?;

    for name in llvm_config("--libnames")?.trim().split(' ') {
        println!("cargo:rustc-link-lib=static={}", parse_library_name(name)?);
    }

    for flag in llvm_config("--system-libs")?.trim().split(' ') {
        let flag = flag.trim_start_matches("-l");

        if flag.starts_with('/') {
            // llvm-config returns absolute paths for dynamically linked libraries.
            let path = Path::new(flag);

            println!(
                "cargo:rustc-link-search={}",
                path.parent().unwrap().display()
            );
            println!(
                "cargo:rustc-link-lib={}",
                parse_library_name(path.file_name().unwrap().to_str().unwrap())?
            );
        } else {
            println!("cargo:rustc-link-lib={}", flag);
        }
    }

    println!("cargo:rustc-link-lib=ffi");

    if let Some(name) = get_system_libcpp() {
        println!("cargo:rustc-link-lib={name}");
    }

    bindgen::builder()
        .header("wrapper.h")
        .clang_arg("-Icc/include")
        .clang_arg(format!("-I{}", llvm_config("--includedir")?))
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap()
        .write_to_file(Path::new(&env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}

fn build_c_library() -> Result<(), Box<dyn Error>> {
    env::set_var("CXXFLAGS", llvm_config("--cxxflags")?);
    env::set_var("CFLAGS", llvm_config("--cflags")?);
    println!("cargo:rustc-link-search={}", &env::var("OUT_DIR")?);

    cc::Build::new()
        .files(
            read_dir("cc/lib")?
                .filter_map(|result| result.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_file() && path.extension().unwrap() == "cpp"),
        )
        .cpp(true)
        .include("cc/include")
        .include(llvm_config("--includedir")?)
        .flag(&llvm_config("--cxxflags")?)
        .flag("-Wno-unused-parameter")
        .std("c++17")
        .compile("CTableGen");

    println!("cargo:rustc-link-lib=static=CTableGen");

    Ok(())
}

fn get_system_libcpp() -> Option<&'static str> {
    if cfg!(target_env = "msvc") {
        None
    } else if cfg!(target_os = "macos") {
        Some("c++")
    } else {
        Some("stdc++")
    }
}

fn llvm_config(argument: &str) -> Result<String, Box<dyn Error>> {
    let prefix = env::var(format!("TABLEGEN_{}0_PREFIX", LLVM_MAJOR_VERSION))
        .map(|path| Path::new(&path).join("bin"))
        .unwrap_or_default();
    let call = format!(
        "{} --link-static {argument}",
        prefix.join("llvm-config").display()
    );

    Ok(str::from_utf8(
        &if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &call]).output()?
        } else {
            Command::new("sh").arg("-c").arg(&call).output()?
        }
        .stdout,
    )?
    .trim()
    .to_string())
}

fn parse_library_name(name: &str) -> Result<&str, String> {
    name.strip_prefix("lib")
        .and_then(|name| name.split('.').next())
        .ok_or_else(|| format!("failed to parse library name: {name}"))
}
