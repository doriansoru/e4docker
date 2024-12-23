use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target == "x86_64-pc-windows-gnu" {
        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("icon.res");

        // Assicurati che il percorso del file `icon.rc` sia corretto
        let icon_rc_path = Path::new("icon.rc");

        // Converte il percorso in una stringa
        let icon_rc_str = icon_rc_path.to_str().unwrap();
        let dest_path_str = dest_path.to_str().unwrap();

        /*println!("cargo:warning=OUT_DIR: {}", out_dir);
        println!("cargo:warning=DEST_PATH: {}", dest_path_str);
        println!("cargo:warning=ICON_RC_PATH: {}", icon_rc_str);*/

        Command::new("x86_64-w64-mingw32-windres")
            .args(&[icon_rc_str, "-O", "coff", "-o", dest_path_str])
            .status()
            .unwrap();

        // Verifica che il file `icon.res` sia stato creato
        /*if dest_path.exists() {
            println!("cargo:warning=icon.res created successfully at {}", dest_path_str);
        } else {
            panic!("icon.res was not created at {}", dest_path_str);
        }*/

        // Passa il percorso del file `icon.res` senza il prefisso `/WIN32RES:`
        println!("cargo:rustc-link-arg={}", dest_path.canonicalize().unwrap().display());
    }
}
