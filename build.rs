extern crate embed_resource;
use std::env;

fn main() {
    set_game_icon();
    assign_cargo_meta_data();
}

fn set_game_icon() {
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        // on windows we will set our game icon as icon for the executable
        embed_resource::compile("build/windows/icon.rc");
    }
}

fn assign_cargo_meta_data() {
    // Read the Cargo.toml metadata
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("Failed to read Cargo metadata");

    // Extract the custom project metadata
    let package_metadata = &metadata
        .packages
        .iter()
        .find(|package| package.name == env::var("CARGO_PKG_NAME").unwrap())
        .expect("Package not found")
        .metadata;

    if let Some(project) = package_metadata.get("project") {
        if let Some(qualifier) = project.get("qualifier").and_then(|v| v.as_str()) {
            println!("cargo:rustc-env=PROJECT_QUALIFIER={}", qualifier);
        }
        if let Some(organization) = project.get("organization").and_then(|v| v.as_str()) {
            println!("cargo:rustc-env=PROJECT_ORGANIZATION={}", organization);
        }
        if let Some(application) = project.get("application").and_then(|v| v.as_str()) {
            println!("cargo:rustc-env=PROJECT_APPLICATION={}", application);
        }
        if let Some(support_link) = project.get("support_link").and_then(|v| v.as_str()) {
            println!("cargo:rustc-env=PROJECT_SUPPORT_LINK={}", support_link);
        }
    }
}
