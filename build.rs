fn main() {
    // GLFW for windows should be put in this folder.
    // You could compile it with a custom directory but this is where GLFW
    // installs itself.
    if cfg!(windows) {
        println!("cargo:rustc-link-search=C:\\Program Files (x86)\\GLFW\\lib");
    }
}
