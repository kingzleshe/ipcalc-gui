fn main() {
    println!("cargo:rerun-if-changed=ui/app-icon.ico");
    println!("cargo:rerun-if-changed=ui/app.slint");

    slint_build::compile("ui/app.slint").expect("failed to compile Slint UI");

    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("ui/app-icon.ico");
        resource.set("FileDescription", "IPCalc");
        resource.set("ProductName", "IPCalc");
        resource
            .compile()
            .expect("failed to compile Windows resources");
    }
}
