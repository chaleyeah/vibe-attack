use std::io::Write;

fn write_temp_yaml(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn config_roundtrip_valid_minimal() {
    let yaml = r#"
ptt:
  key: KEY_F13
timing:
  dwell_ms: 50
  gap_ms: 30
"#;
    let f = write_temp_yaml(yaml);
    let cfg = hd_linux_voice::config::load(Some(f.path())).expect("valid config must load");
    assert_eq!(cfg.ptt.key, "KEY_F13");
    assert_eq!(cfg.timing.dwell_ms, 50);
    assert_eq!(cfg.timing.gap_ms, 30);
    assert!(cfg.macros.is_empty());
}

#[test]
fn config_rejects_unknown_fields() {
    let yaml = r#"
ptt:
  key: KEY_F13
timing:
  dwell_ms: 50
  gap_ms: 30
unknown_field: bad
"#;
    let f = write_temp_yaml(yaml);
    let result = hd_linux_voice::config::load(Some(f.path()));
    assert!(result.is_err(), "unknown_field must cause an error");
}

#[test]
fn config_wrong_type_returns_err_not_panic() {
    let yaml = r#"
ptt:
  key: KEY_F13
timing:
  dwell_ms: "fast"
  gap_ms: 30
"#;
    let f = write_temp_yaml(yaml);
    let result = hd_linux_voice::config::load(Some(f.path()));
    assert!(result.is_err(), "wrong type must fail with Err");
}

#[test]
fn config_missing_file_returns_err() {
    let result = hd_linux_voice::config::load(
        Some(std::path::Path::new("/tmp/nonexistent_hd_config_xyz.yaml"))
    );
    assert!(result.is_err());
    let msg = format!("{:#}", result.unwrap_err());
    assert!(msg.contains("not found") || msg.contains("No such file"),
        "Error message should mention file not found, got: {msg}");
}

#[test]
fn config_optional_macros_empty_is_valid() {
    let yaml = r#"
ptt:
  key: KEY_GRAVE
timing:
  dwell_ms: 60
  gap_ms: 25
"#;
    let f = write_temp_yaml(yaml);
    let cfg = hd_linux_voice::config::load(Some(f.path())).expect("macros optional");
    assert!(cfg.macros.is_empty());
}

#[test]
fn config_per_key_dwell_override() {
    let yaml = r#"
ptt:
  key: KEY_F13
timing:
  dwell_ms: 50
  gap_ms: 30
macros:
  - name: test
    keys:
      - key: KEY_UP
      - key: KEY_DOWN
        dwell_ms: 100
"#;
    let f = write_temp_yaml(yaml);
    let cfg = hd_linux_voice::config::load(Some(f.path())).unwrap();
    let keys = &cfg.macros[0].keys;
    assert_eq!(keys[0].dwell_ms, None);
    assert_eq!(keys[1].dwell_ms, Some(100));
}
