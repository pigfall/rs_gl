use gl_generator::{Api,Fallbacks,Registry,Profile};
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main(){
    let dest_dirpath =  PathBuf::from(&env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=build.rs");

    let mut file_to_write = File::create(&dest_dirpath.join("gl_bindings.rs")).unwrap();
    let mut file_backup = File::create(PathBuf::from("gl_bindings_backup.rs")).unwrap();
    Registry::new(
        Api::Gl,
        (4,5),
        Profile::Core,
        Fallbacks::All,
        []
        )
        .write_bindings(
            gl_generator::StructGenerator,
            &mut file_to_write,
            )
        .unwrap();

    Registry::new(
        Api::Gl,
        (4,5),
        Profile::Core,
        Fallbacks::All,
        []
        )
        .write_bindings(
            gl_generator::StructGenerator,
            &mut file_backup,
            )
        .unwrap();
}
