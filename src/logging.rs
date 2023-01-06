use indoc::indoc;
use log4rs;

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
        std::fs::write(log_file_path_str, default_config)
            .expect("Unable to write default logfile");
    }

    log4rs::init_file(log_file_path, Default::default()).unwrap();
}
