use std::path::{Path, PathBuf};
use std::fs;
use std::io::Cursor;

fn main() {
    // We check for Windows target during compilation
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        setup_windows_deps();
    }
    tauri_build::build();
}

fn setup_windows_deps() {
    let base_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let res_dir = base_dir.join("resources");
    
    // 1. Setup Git (MinGit)
    let git_dir = res_dir.join("git");
    if !git_dir.exists() {
        println!("cargo:warning=Downloading MinGit for Windows...");
        fs::create_dir_all(&git_dir).unwrap();
        let url = "https://github.com/git-for-windows/git/releases/download/v2.44.0.windows.1/MinGit-2.44.0-64-bit.zip";
        download_and_extract_zip(url, &git_dir);
    }

    // 2. Setup GnuPG
    let gpg_dir = res_dir.join("gnupg");
    if !gpg_dir.exists() || fs::read_dir(&gpg_dir).map(|mut d| d.next().is_none()).unwrap_or(true) {
        println!("cargo:warning=Downloading GnuPG for Windows...");
        let _ = fs::create_dir_all(&gpg_dir);
        // Using the .tar.xz version which is available
        let url = "https://gnupg.org/ftp/gcrypt/binary/gnupg-w32-2.4.5_20240307.tar.xz";
        download_and_extract_tar_xz(url, &gpg_dir);
    }
}

fn download_and_extract_zip(url: &str, dest: &Path) {
    let response = reqwest::blocking::get(url).expect("Failed to download dependency");
    let content = Cursor::new(response.bytes().expect("Failed to read response bytes"));
    
    let mut archive = zip::ZipArchive::new(content).expect("Failed to open zip archive");
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }
}

fn download_and_extract_tar_xz(url: &str, dest: &Path) {
    let response = reqwest::blocking::get(url).expect("Failed to download dependency");
    let bytes = response.bytes().expect("Failed to read response bytes");
    let xz_decoder = xz2::read::XzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(xz_decoder);
    
    archive.unpack(dest).expect("Failed to unpack tar.xz archive");
}
