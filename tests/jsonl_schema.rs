#[test]
fn jsonl_event_has_required_fields_and_stable_keys() {
    let evt = hd_linux_voice::pipeline::jsonl::JsonlEvent::Utterance {
        utterance_id: 42,
        wall_time_ms: 1_700_000_000_000,
        mono_ms: 123,
        text: "hello",
        audio_ms: 640,
    };

    let line = serde_json::to_string(&evt).expect("serialize json");
    assert!(
        !line.contains('\n'),
        "event serialization must be a single-line JSON object"
    );
    let v: serde_json::Value = serde_json::from_str(&line).expect("parse json");

    assert_eq!(v["type"], "utterance");
    assert_eq!(v["utterance_id"], 42);
    assert_eq!(v["text"], "hello");
    assert!(v.get("wall_time_ms").is_some(), "wall_time_ms missing");
    assert!(v.get("mono_ms").is_some(), "mono_ms missing");
    assert!(v.get("audio_ms").is_some(), "audio_ms missing");
    assert!(
        v["wall_time_ms"].as_u64().is_some(),
        "wall_time_ms must be u64"
    );
    assert!(v["mono_ms"].as_u64().is_some(), "mono_ms must be u64");
    assert!(v["audio_ms"].as_u64().is_some(), "audio_ms must be u64");
}

#[test]
fn jsonl_timing_fields_are_non_negative() {
    let evt = hd_linux_voice::pipeline::jsonl::JsonlEvent::Utterance {
        utterance_id: 0,
        wall_time_ms: 0,
        mono_ms: 0,
        text: "",
        audio_ms: 0,
    };

    let line = serde_json::to_string(&evt).expect("serialize");
    let v: serde_json::Value = serde_json::from_str(&line).expect("parse json");

    // Represented as u64 in the JSON schema, so negative values are impossible.
    assert!(v["wall_time_ms"].as_u64().is_some());
    assert!(v["mono_ms"].as_u64().is_some());
    assert!(v["audio_ms"].as_u64().is_some());
}

