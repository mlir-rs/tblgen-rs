use std::{
    env,
    error::Error,
    ffi::OsStr,
    fs::read_dir,
    path::Path,
    process::{Command, exit},
    str,
};

const LLVM_MAJOR_VERSION: usize = if cfg!(feature = "llvm16-0") {
    16
} else if cfg!(feature = "llvm17-0") {
    17
} else if cfg!(feature = "llvm18-0") {
    18
} else if cfg!(feature = "llvm19-0") {
    19
} else if cfg!(feature = "llvm20-0") {
    20
} else if cfg!(feature = "llvm21-0") {
    21
} else {
    22
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let version = llvm_config(false, "--version")?;

    if !version.starts_with(&format!("{LLVM_MAJOR_VERSION}.")) {
        return Err(format!(
            "failed to find correct version ({LLVM_MAJOR_VERSION}.x.x) of llvm-config (found {version})",
        )
        .into());
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=cc");
    println!(
        "cargo:rustc-link-search={}",
        llvm_config(false, "--libdir")?
    );

    build_c_library()?;

    let link_static = resolve_link_mode()?;

    // When using the shared libLLVM, it may not export all C++ symbols that
    // libCTableGen.a references (e.g. VarInit::getName()). Link the available
    // static component libs to cover the gap.
    if !link_static {
        let libdir = llvm_config(false, "--libdir")?;
        for lib in &["LLVMTableGen", "LLVMSupport", "LLVMDemangle"] {
            if Path::new(&format!("{}/lib{}.a", libdir, lib)).exists() {
                println!("cargo:rustc-link-lib=static={}", lib);
            }
        }
    }

    for name in llvm_config(link_static, "--libnames")?
        .trim()
        .split(' ')
        .filter(|s| !s.is_empty())
    {
        let link_type = if name.ends_with(".a") {
            "static="
        } else {
            "dylib="
        };
        println!(
            "cargo:rustc-link-lib={}{}",
            link_type,
            parse_library_name(name)?
        );
    }

    for flag in llvm_config(link_static, "--system-libs")?
        .trim()
        .split(' ')
        .filter(|s| !s.is_empty())
    {
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

    if let Some(name) = get_system_libcpp() {
        println!("cargo:rustc-link-lib={name}");
    }

    bindgen::builder()
        .header("wrapper.h")
        .clang_arg("-Icc/include")
        .clang_arg(format!("-I{}", llvm_config(false, "--includedir")?))
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?
        .write_to_file(Path::new(&env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}

fn filter_fortify_source(flags: &str) -> String {
    flags
        .split_whitespace()
        .filter(|flag| !flag.starts_with("-D_FORTIFY_SOURCE"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_c_library() -> Result<(), Box<dyn Error>> {
    let raw_cxxflags = llvm_config(false, "--cxxflags")?;
    let raw_cflags = llvm_config(false, "--cflags")?;

    // The cc crate does not add -O when OPT_LEVEL=0, so -D_FORTIFY_SOURCE (which
    // glibc requires to be paired with optimization) causes a #warning that
    // -Werror turns into a hard error. Strip it only when compiling without
    // optimization; otherwise keep it for the runtime hardening it provides.
    let (cxxflags, cflags) = if env::var("OPT_LEVEL").as_deref() == Ok("0") {
        (
            filter_fortify_source(&raw_cxxflags),
            filter_fortify_source(&raw_cflags),
        )
    } else {
        (raw_cxxflags, raw_cflags)
    };
    unsafe { env::set_var("CXXFLAGS", cxxflags) };
    unsafe { env::set_var("CFLAGS", cflags) };

    cc::Build::new()
        .cpp(true)
        .files(
            read_dir("cc/lib")?
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|entry| entry.path())
                .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("cpp"))),
        )
        .include("cc/include")
        .include(llvm_config(false, "--includedir")?)
        .flag(if cfg!(target_env = "msvc") {
            "/WX"
        } else {
            "-Werror"
        })
        .flag(if cfg!(target_env = "msvc") {
            "/W4"
        } else {
            "-Wall"
        })
        .flag(if cfg!(target_env = "msvc") {
            "" // /W4 already covers extras on MSVC
        } else {
            "-Wno-unused-parameter"
        })
        .std("c++17")
        .compile("CTableGen");

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

fn resolve_link_mode() -> Result<bool, Box<dyn Error>> {
    let available = llvm_config(true, "--libnames").is_ok();
    if cfg!(feature = "force-static") {
        if !available {
            return Err(
                "LLVM static libraries not available but `force-static` feature is enabled".into(),
            );
        }
        Ok(true)
    } else {
        Ok(available)
    }
}

fn llvm_config(link_static: bool, argument: &str) -> Result<String, Box<dyn Error>> {
    let prefix = env::var(format!("TABLEGEN_{}0_PREFIX", LLVM_MAJOR_VERSION))
        .map(|path| Path::new(&path).join("bin"))
        .unwrap_or_default();
    let static_flag = if link_static { "--link-static " } else { "" };
    let call = format!(
        "{} {static_flag}{argument}",
        prefix.join("llvm-config").display()
    );

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", &call]).output()?
    } else {
        Command::new("sh").arg("-c").arg(&call).output()?
    };

    if !output.status.success() {
        return Err(format!("llvm-config {static_flag}{argument} failed").into());
    }

    Ok(str::from_utf8(&output.stdout)?.trim().to_string())
}

fn parse_library_name(name: &str) -> Result<&str, String> {
    name.strip_prefix("lib")
        .and_then(|name| name.split('.').next())
        .ok_or_else(|| format!("failed to parse library name: {name}"))
}
