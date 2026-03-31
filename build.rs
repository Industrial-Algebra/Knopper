fn main() {
    if std::env::var_os("CARGO_FEATURE_NOTCURSES").is_some() {
        pkg_config::Config::new()
            .atleast_version("3.0.11")
            .probe("notcurses")
            .expect("pkg-config couldn't find the notcurses library");
    }
}
