use vergen_gitcl::{BuildBuilder, CargoBuilder, Emitter, GitclBuilder, RustcBuilder, SysinfoBuilder};

fn get_optional_env(key: &str) -> Option<String> {
    if let Ok(val) = std::env::var(key) {
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}

fn main() {
    //
    // Rerun cargo if this file is changed:
    //
    println!("cargo:rerun-if-changed=build.rs");

    if let Some(vcpkg_install_lib_path) = get_optional_env("VCPKG_INSTALL_LIB_PATH") {
        //
        // Set RPATH so that environment variable LD_LIBRARY_PATH is not required
        //
        println!("cargo:rustc-link-arg=-Wl,-rpath,{vcpkg_install_lib_path}");
    }

    //
    // Rerun cargo if one of the internationalization files change:
    //
    println!("cargo:rerun-if-changed=i18n.toml");
    println!("cargo:rerun-if-changed=i18n/en/mxl_plyr.ftl");
    println!("cargo:rerun-if-changed=i18n/de/mxl_plyr.ftl");

    //
    // Add link search path to find libararies in the installation directory:
    //
    println!("cargo:rustc-link-search=native=../lib");

    //
    // Fix problem with Apps on macOS, that should inherent the environment variables from the terminal:
    //
    #[cfg(target_os = "macos")]
    if let Ok(rpath) = env::var("RPATH") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", rpath);
    }

    //
    // Compile GTK resource file:
    //
    glib_build_tools::compile_resources(
        &["data/resources/icons"],
        "data/resources/resources.gresource.xml",
        "resources.gresource",
    );
    println!("cargo:rerun-if-changed=data/resources/resources.gresource.xml");
    println!("cargo:rerun-if-changed=data/resources/icons/com.x-software.mxl.plyr.svg");

    //
    // Provide build and repository information for about.rs:
    //
    Emitter::default()
        .add_instructions(&BuildBuilder::all_build().unwrap())
        .unwrap()
        .add_instructions(&CargoBuilder::all_cargo().unwrap())
        .unwrap()
        .add_instructions(&GitclBuilder::all_git().unwrap())
        .unwrap()
        .add_instructions(&RustcBuilder::all_rustc().unwrap())
        .unwrap()
        .add_instructions(&SysinfoBuilder::all_sysinfo().unwrap())
        .unwrap()
        .emit()
        .unwrap();
}
