extern crate tempdir;

use indoc::indoc;
use tempdir::TempDir;

pub fn setup_logging() {
    let mut log_file_path = std::env::current_exe().unwrap();
    log_file_path.pop();
    log_file_path.push("log4rs.yaml");
    let default_config = indoc! {r#"
appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l} - {f}:{L} - {m}{n})}"
  file_logger:
    kind: rolling_file
    path: "logs/mtfg-rs.log"
    encoder:
      pattern: "{d(%Y-%m-%d %H:%M:%S)(utc)} - {l} - {f}:{L} - {m}{n}"
    policy:
      trigger:
        kind: size
        limit: 1Mb
      roller:
        kind: fixed_window
        base: 1
        count: 3
        pattern: "logs/mtfg-rs_{}.log"
root:
  level: info
  appenders:
    - stdout
    - file_logger
"#};

    let log_file_path_str = log_file_path.as_os_str();
    if !std::path::Path::new(log_file_path_str).exists() {
        // TODO nix can not write to the application directory
        // Ugly workaround below
        let tmp_dir = TempDir::new("mtfg-rs").expect("Failed to create tmp log configuration");
        let tmp_log_config = tmp_dir.path().join("log4rs.yaml");
        std::fs::write(&tmp_log_config, default_config).expect("Unable to write default logfile");
        log4rs::init_file(tmp_log_config, Default::default()).unwrap();
        return;
    }

    log4rs::init_file(log_file_path, Default::default()).unwrap();
}
