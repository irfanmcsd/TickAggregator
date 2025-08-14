fn main() {
     // Only run when targeting Windows
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/app.ico"); // path to your icon
        res.compile().unwrap();
    }
}