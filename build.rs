// When the `notcurses` feature is enabled, assert a system notcurses
// development library is available via pkg-config. The `notcurses` crate
// links against it.
//
// CI installs `libnotcurses-dev`; local developers must install it
// (e.g. `apt install libnotcurses-dev` or the equivalent for their system)
// before building with `--features notcurses`.
fn main() {
    if std::env::var_os("CARGO_FEATURE_NOTCURSES").is_some() {
        pkg_config::Config::new()
            .atleast_version("3.0.11")
            .probe("notcurses")
            .expect(
                "pkg-config couldn't find notcurses >= 3.0.11. \
                 Install the system development library \
                 (e.g. `apt install libnotcurses-dev`) before building with \
                 the `notcurses` feature.",
            );
    }
}
