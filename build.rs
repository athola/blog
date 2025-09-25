use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=style/tailwind.css");

    // Ensure the target directory exists
    let site_pkg_dir = Path::new("target/site/pkg");
    if !site_pkg_dir.exists() {
        fs::create_dir_all(site_pkg_dir).expect("Failed to create site/pkg directory");
    }

    // Copy the CSS file if it exists
    let css_source = Path::new("target/tmp/tailwind.css");
    let css_dest = Path::new("target/site/pkg/blog.css");

    if css_source.exists() {
        fs::copy(css_source, css_dest).expect("Failed to copy CSS file");
        println!("Copied CSS from {:?} to {:?}", css_source, css_dest);
    } else {
        println!("Source CSS file not found at {:?}", css_source);
    }

    // Copy favicon to site root
    let favicon_source = Path::new("public/favicon.ico");
    let favicon_dest = Path::new("target/site/favicon.ico");

    if favicon_source.exists() {
        fs::copy(favicon_source, favicon_dest).expect("Failed to copy favicon file");
        println!("Copied favicon to site root");
    } else {
        println!("Favicon not found at {:?}", favicon_source);
    }

    // Create symlink for WASM file to fix the blog_bg.wasm issue
    let wasm_dest = Path::new("target/site/pkg/blog_bg.wasm");
    let wasm_source = Path::new("blog.wasm");

    // Remove existing destination if it exists
    if wasm_dest.exists() {
        fs::remove_file(wasm_dest).expect("Failed to remove existing blog_bg.wasm");
    }

    // Change to the pkg directory to create relative symlink
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(site_pkg_dir).expect("Failed to change to pkg directory");

    // Create relative symlink
    #[cfg(unix)]
    {
        match std::os::unix::fs::symlink(wasm_source, "blog_bg.wasm") {
            Ok(_) => println!("Successfully created WASM symlink"),
            Err(e) => println!("Failed to create WASM symlink: {}", e),
        }
    }

    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_file(wasm_source, "blog_bg.wasm") {
            Ok(_) => println!("Successfully created WASM symlink"),
            Err(e) => println!("Failed to create WASM symlink: {}", e),
        }
    }

    // Change back to original directory
    std::env::set_current_dir(original_dir).expect("Failed to change back to original directory");

    println!("Created WASM symlink blog_bg.wasm -> blog.wasm");
}
