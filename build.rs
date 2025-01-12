use edres::{generate_enum_from_source, EnumOptions, Format, Options};
use std::env;
use std::fs;
use std::path::PathBuf;
include!("./src/utils/config.rs");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=config.rs");
    // Read the TOML file
    let roles_toml = fs::read_to_string("config.toml").expect("Unable to read roles.toml");
    let convert_to_toml: ConfigData = toml::from_str(&roles_toml).unwrap();
    let necessary = toml::to_string(&convert_to_toml.roles.public).unwrap();

    // Generate the enum from the TOML file content
    let generated_code = generate_enum_from_source(
        &necessary,
        "RoleEnum",
        Format::Toml,
        &Options {
            enums: EnumOptions {
                impl_display: true,
                impl_from_str: true,
                derived_traits: vec![
                    "poise::ChoiceParameter".into(),
                    "serde::Serialize".into(),
                    "Eq".into(),
                    "Hash".into(),
                    "PartialEq".into(),
                ]
                .into(),
                ..EnumOptions::minimal()
            },
            ..Options::default()
        },
    )
    .unwrap();

    // Write the generated code to a file in the OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("generated_roles.rs"), generated_code)
        .expect("Unable to write generated roles to file");
}
