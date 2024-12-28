use base64::{Engine, engine::general_purpose};
use std::{io::Write, path::PathBuf};

const GENERIC_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAABhGlDQ1BJQ0MgcHJvZmlsZQAAKJF9
kT1Iw0AYht+miiItgnYQcchQnexiRRxrFYpQIdQKrTqYXPoHTRqSFBdHwbXg4M9i1cHFWVcHV0EQ
/AFxdnBSdJESv0sKLWK847iH97735e47QGhWmWb1JABNt81MKinm8qti3ytCGKIZRlxmljEnSWn4
jq97BPh+F+NZ/nV/jrBasBgQEIkTzDBt4g3imU3b4LxPHGFlWSU+J5406YLEj1xXPH7jXHJZ4JkR
M5uZJ44Qi6UuVrqYlU2NeJo4qmo65Qs5j1XOW5y1ap2178lfGCroK8tcpzWGFBaxBAkiFNRRQRU2
YrTrpFjI0HnSxz/q+iVyKeSqgJFjATVokF0/+B/87q1VjE95SaEk0PviOB/jQN8u0Go4zvex47RO
gOAzcKV3/LUmMPtJeqOjRY+AwW3g4rqjKXvA5Q4w8mTIpuxKQVpCsQi8n9E35YHhW2Bgzetb+xyn
D0CWepW+AQ4OgYkSZa/7vLu/u2//1rT79wO1iXLBTW89ywAAAAZiS0dEAP8A/wD/oL2nkwAAAAlw
SFlzAAAuIwAALiMBeKU/dgAAAAd0SU1FB+gMEhIPHIUYdsQAAAHRSURBVFjD7Zc/axRhEIef322s
PMU6+AHUzsomCN4GMbh3t5cirSi2YiFIQEgOCTY2foGgjYVN3Nx6aMhGDtRWwSrYe3WCsZHsToqs
yeGFZPcuRwLuVPuP931mZued36jT6RgnaCVO2MYOe1mdbEyYJeN5FjSx+SZ4fbFcLi9m+V4HpaDq
+vcMXgDnB3QsNtnzdrQ8mzsFtyenpwwWh9gcwJFp1nP9Vu4UyJIngIBtTE2U/MgUSnHNTI/SRGyA
LgA1z/Vb79aCWmYAg6sChK2GH4OFrC57bn1mbw2zpqTHwPhREH0pEJzdBSn9GjT+krqK4wrQTR/V
PNcPRlqGUqnbUwqXwk64rji+0QPhe5XGg5EBhNHbz8BmGsb5qlt/b87YgtB3gz8p2f1c50BeM3go
eAk4hqbAsPSPTsGujPQkbK8Fr0x2C/gEbO2D2XZ6eWakEQBoR8sr/RXSWAKbOZW9oAAoAAqAAqAA
OH0ABr93O2dy7tjECslfgbt1tCiFb8CEoZtexZ9Dtj6cSNBlpLn07itw/XBRKmvKtAo4iKc9cmJQ
9/e1qpJn/wKUDujnkcHdPXl1PLZhcKcdtT5kmoyGGc36xarzM4yWvuQazf6rMtwBbWiehsGr5+cA
AAAASUVORK5CYII=";

pub fn create_generic_button(destination: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let img64 = GENERIC_PNG.replace("\n", "");
    let bytes = general_purpose::STANDARD.decode(img64)?;

    // Create a file on destination
    let mut file = std::fs::File::create(destination)?;

    // Write the bytes on file
    file.write_all(&bytes)?;

    Ok(())
}

pub fn get_package_config_dir() -> PathBuf {
    // Get the package name
    let package_name = env!("CARGO_PKG_NAME");

    let config_dir = dirs::config_dir().expect("Cannot create the configuration directory.");

    // Create the path of the configuration directory for this app
    let project_config_dir = config_dir.join(package_name);
    let assets_dir = project_config_dir.join("assets");

    // Create this app configuration directory if it does not exist
    if !project_config_dir.exists() {
        // Create the project configuration directory
        std::fs::create_dir_all(&project_config_dir)
            .expect("Cannot create the project config directory.");
        // Create the assets directory
        std::fs::create_dir_all(&assets_dir)
                .expect("Cannot create assets config directory.");
    }

    // Generic button png file
    let mut generic_png =  assets_dir.join("generic");
    generic_png.set_extension("png");
    if !generic_png.exists()  {
        match create_generic_button(&generic_png) {
            Ok(_) => {},
            Err(e) => {
                panic!("Cannot create {}: {}", generic_png.display(), e);
            },
        }
    }

    // Generic button conf file
    let mut generic_conf =  project_config_dir.join("generic");
    generic_conf.set_extension("conf");
    if !generic_conf.exists() {
        // Create generic.conf
        let mut file = std::fs::File::create(&generic_conf).expect("Cannot create generic.conf");
        file.write_all(b"[button]
arguments=
icon=generic.png
command=/usr/bin/generic").expect("Cannot write on generic.conf");
    }

    // App conf file
    let mut e4docker_conf = project_config_dir.join(package_name);
    e4docker_conf.set_extension("conf");
    if !e4docker_conf.exists() {
        // Create generic.conf
        let mut file = std::fs::File::create(&e4docker_conf).expect("Cannot create e4docker.conf");
        file.write_all(b"[e4docker]
number_of_buttons=1
frame_margin=10
margin_between_buttons=20
icon_width=32
icon_height=32
[buttons]
button1=generic").expect("Cannot write on e4docker.conf");
    }

    project_config_dir
}

pub fn get_package_assets_dir() -> PathBuf {
    get_package_config_dir().join("assets")
}

pub fn get_generic_icon() -> PathBuf {
    get_package_assets_dir().join("generic.png")
}
