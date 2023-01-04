use indoc::indoc;
use log4rs;

pub fn setup_logging() {
    let log_file_path = "log4rs.yaml";
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

    if !std::path::Path::new(log_file_path).exists() {
        std::fs::write(log_file_path, default_config)
            .expect(format!("Unable to write default logfile: {log_file_path}").as_str());
    }

    log4rs::init_file(log_file_path, Default::default()).unwrap();
}
