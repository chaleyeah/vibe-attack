// ConfigApp.jsx — main config window

const ConfigApp = ({ status = "running", iconSet = "svg", initialNav = "devices" }) => {
  const [nav, setNav] = React.useState(initialNav);
  const [mode, setMode] = React.useState("ptt");
  const [threshold, setThreshold] = React.useState(0.8);
  const [device, setDevice] = React.useState("system");
  const [wakeWord, setWakeWord] = React.useState("hey diver");
  const [activeProfile, setActiveProfile] = React.useState("ImportTest");
  const [autostart, setAutostart] = React.useState(true);
  const [pttKey, setPttKey] = React.useState("KEY_LEFTCTRL");

  // animate mic level
  const [micLvl, setMicLvl] = React.useState(0.32);
  React.useEffect(() => {
    if (status !== "running") return;
    const t = setInterval(() => {
      setMicLvl(0.18 + Math.random() * 0.55);
    }, 220);
    return () => clearInterval(t);
  }, [status]);

  return (
    <>
      <AppHeader status={status} iconSet={iconSet}>
        <button className="btn btn-ghost btn-icon" title="Save"><Icon name="save" set={iconSet} /></button>
      </AppHeader>

      <div className="app-body">
        <LeftRail active={nav} onChange={setNav} iconSet={iconSet} />

        <main className="main">
          {status === "error" && (
            <div style={{ padding: "16px 24px 0" }}>
              <Banner kind="error" title="DAEMON UNREACHABLE" actions={
                <>
                  <button className="btn btn-primary"><Icon name="refresh" set={iconSet} size={12} />Reconnect</button>
                  <button className="btn">View logs</button>
                </>
              }>
                Failed to connect to <code>/run/user/1000/vibe-attack/vibe-attack.sock</code>. Edits will queue and apply once the daemon is back online.
              </Banner>
            </div>
          )}

          {nav === "devices" && (
            <DevicesPane
              device={device} setDevice={setDevice}
              micLvl={micLvl} status={status}
              iconSet={iconSet}
            />
          )}
          {nav === "voice" && (
            <VoicePane
              mode={mode} setMode={setMode}
              threshold={threshold} setThreshold={setThreshold}
              wakeWord={wakeWord} setWakeWord={setWakeWord}
              iconSet={iconSet}
            />
          )}
          {nav === "packs" && (
            <PacksPane activeProfile={activeProfile} setActiveProfile={setActiveProfile} iconSet={iconSet} />
          )}
          {nav === "hotkeys" && (
            <HotkeysPane pttKey={pttKey} setPttKey={setPttKey} iconSet={iconSet} />
          )}
          {nav === "advanced" && (
            <AdvancedPane autostart={autostart} setAutostart={setAutostart} iconSet={iconSet} />
          )}
        </main>
      </div>

      <StatusFooter status={status} micLevel={micLvl} />
    </>
  );
};

const DevicesPane = ({ device, setDevice, micLvl, status, iconSet }) => (
  <>
    <Section
      title="Audio Input"
      sub="Select the capture device the daemon listens on"
      actions={<button className="btn"><Icon name="refresh" set={iconSet} size={12} />Rescan</button>}
    >
      <FieldRow label="Input device" required help="ALSA / PulseAudio / PipeWire device name. Hot-pluggable.">
        <select className="select" value={device} onChange={(e) => setDevice(e.target.value)}>
          <option value="system">&lt;system default&gt; — alsa_input.usb-Blue_Yeti</option>
          <option value="yeti">Blue Yeti — USB Audio</option>
          <option value="hdmic">Built-in Microphone (Internal)</option>
          <option value="bt">Sony WH-1000XM4 (handsfree)</option>
        </select>
      </FieldRow>

      <FieldRow label="Sample rate" help="Lower = less CPU. 16 kHz is sufficient for whisper.">
        <select className="select" defaultValue="16000">
          <option>8000 Hz</option>
          <option>16000 Hz</option>
          <option>22050 Hz</option>
          <option>44100 Hz</option>
          <option>48000 Hz</option>
        </select>
      </FieldRow>

      <FieldRow label="Live monitor" help="Realtime input level. Use this to verify the device is actually receiving audio.">
        <div style={{ background: "var(--bg-extreme)", border: "1px solid var(--stroke)", borderRadius: 3, padding: "10px 12px" }}>
          <LedMeter level={status === "muted" ? 0 : micLvl} segments={32} />
          <div style={{ display: "flex", justifyContent: "space-between", marginTop: 6, fontSize: 10, color: "var(--fg-faint)", letterSpacing: "0.06em" }}>
            <span>−∞</span>
            <span>−24</span>
            <span>−12</span>
            <span>−6</span>
            <span>0 dB</span>
          </div>
        </div>
      </FieldRow>
    </Section>

    <Section title="Voice Activity (VAD)" sub="Silero-based VAD, runs before STT to drop silence">
      <FieldRow label="Sensitivity" help="0.30 picks up whispers, 0.85 only clear speech.">
        <Slider value={0.55} />
      </FieldRow>
      <FieldRow label="Min speech" help="Discard utterances shorter than this many ms.">
        <Slider value={250} min={50} max={1000} suffix=" ms" />
      </FieldRow>
    </Section>
  </>
);

const VoicePane = ({ mode, setMode, threshold, setThreshold, wakeWord, setWakeWord, iconSet }) => (
  <>
    <Section title="Trigger Mode" sub="When should the daemon transcribe what you say?">
      <FieldRow label="Mode">
        <RadioGroup
          value={mode}
          onChange={setMode}
          options={[
            { value: "ptt", label: "Push-to-talk" },
            { value: "wake", label: "Wake word" },
            { value: "always", label: "Always listening" },
          ]}
        />
      </FieldRow>

      {mode === "ptt" && (
        <FieldRow label="PTT key" help="Held to capture. Released to commit transcript.">
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <span className="kbd">Left Ctrl</span>
            <span style={{ color: "var(--fg-faint)", fontSize: 11 }}>KEY_LEFTCTRL</span>
            <div style={{ flex: 1 }} />
            <button className="btn"><Icon name="edit" set={iconSet} size={12} />Rebind</button>
          </div>
        </FieldRow>
      )}

      {mode === "wake" && (
        <FieldRow label="Wake phrase" help="Lowercase. Sensitivity tuned by confidence threshold below.">
          <input className="input" value={wakeWord} onChange={(e) => setWakeWord(e.target.value)} />
        </FieldRow>
      )}
    </Section>

    <Section title="Whisper Model" sub="Local STT — no audio leaves your machine">
      <FieldRow label="Model file" help="ggml-format Whisper model. Tiny ≈ 75 MB, Base ≈ 142 MB.">
        <div style={{ display: "flex", gap: 8 }}>
          <input className="input" defaultValue="~/.local/share/vibe-attack/models/ggml-tiny.en.bin" />
          <button className="btn"><Icon name="folder" set={iconSet} size={12} />Browse</button>
        </div>
      </FieldRow>

      <FieldRow label="Confidence threshold" help="Reject transcripts below this score. Higher = fewer false fires.">
        <Slider value={threshold} />
      </FieldRow>

      <FieldRow label="Language">
        <select className="select" defaultValue="en">
          <option value="en">English</option>
          <option value="auto">Auto-detect</option>
          <option value="de">Deutsch</option>
          <option value="es">Español</option>
          <option value="ja">日本語</option>
        </select>
      </FieldRow>
    </Section>
  </>
);

const PacksPane = ({ activeProfile, setActiveProfile, iconSet }) => (
  <>
    <Section title="Active Profile" sub="Which voice pack the daemon currently routes phrases through" actions={
      <>
        <button className="btn"><Icon name="upload" set={iconSet} size={12} />Import</button>
        <button className="btn btn-primary"><Icon name="plus" set={iconSet} size={12} />New profile</button>
      </>
    }>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: 8 }}>
        {[
          { id: "ImportTest",   macros: 7, mode: "PTT" },
          { id: "Stratagems",   macros: 24, mode: "WAKE" },
          { id: "Squad Comms",  macros: 12, mode: "PTT" },
          { id: "Sandbox",      macros: 3, mode: "ALWAYS" },
        ].map((p) => (
          <button
            key={p.id}
            onClick={() => setActiveProfile(p.id)}
            className="list-row"
            style={{
              padding: "12px 14px",
              background: activeProfile === p.id ? "var(--accent-faint)" : "var(--bg-faint)",
              borderColor: activeProfile === p.id ? "var(--accent-line)" : "var(--stroke)",
              borderStyle: "solid",
            }}
            aria-selected={activeProfile === p.id ? "true" : "false"}
          >
            <Icon name={activeProfile === p.id ? "radio" : "target"} set={iconSet} color={activeProfile === p.id ? "var(--accent)" : "var(--fg-faint)"} />
            <div style={{ flex: 1, textAlign: "left" }}>
              <div style={{ fontSize: 13, color: "var(--fg-strong)" }}>{p.id}</div>
              <div style={{ fontSize: 10, color: "var(--fg-faint)", letterSpacing: "0.08em", marginTop: 2 }}>
                {p.macros} MACROS · {p.mode}
              </div>
            </div>
            {activeProfile === p.id && <span className="tag" data-tone="accent">ACTIVE</span>}
          </button>
        ))}
      </div>
    </Section>
    <Section title="Pack Editor" sub={`Editing — ${activeProfile}`}>
      <div style={{ display: "flex", gap: 8 }}>
        <button className="btn"><Icon name="edit" set={iconSet} size={12} />Open editor</button>
        <button className="btn"><Icon name="download" set={iconSet} size={12} />Export</button>
        <div style={{ flex: 1 }} />
        <button className="btn btn-danger"><Icon name="trash" set={iconSet} size={12} />Delete pack</button>
      </div>
    </Section>
  </>
);

const HotkeysPane = ({ pttKey, iconSet }) => (
  <Section title="Global Hotkeys" sub="Captured by the daemon's evdev listener. Active in-game even when window is unfocused.">
    {[
      { name: "Push-to-talk", binding: "Left Ctrl", code: "KEY_LEFTCTRL" },
      { name: "Toggle mute", binding: "Pause", code: "KEY_PAUSE" },
      { name: "Cycle profile", binding: "Ctrl + F9", code: "KEY_F9" },
      { name: "Pause daemon", binding: "Ctrl + Alt + V", code: "KEY_V" },
      { name: "Open config", binding: "— unbound —", code: null },
    ].map((h) => (
      <div key={h.name} className="field-row" style={{ borderBottom: "1px solid var(--stroke-faint)" }}>
        <div className="field-label" style={{ textTransform: "none", fontSize: 12, letterSpacing: 0, color: "var(--fg)" }}>{h.name}</div>
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          {h.code ? <span className="kbd">{h.binding}</span> : <span style={{ color: "var(--fg-dim)", fontSize: 11 }}>{h.binding}</span>}
          {h.code && <span style={{ color: "var(--fg-faint)", fontSize: 10, letterSpacing: "0.08em" }}>{h.code}</span>}
          <div style={{ flex: 1 }} />
          <button className="btn btn-ghost"><Icon name="edit" set={iconSet} size={12} />Rebind</button>
        </div>
      </div>
    ))}
  </Section>
);

const AdvancedPane = ({ autostart, setAutostart, iconSet }) => (
  <>
    <Section title="Daemon">
      <div className="field-row">
        <div className="field-label">Autostart on login</div>
        <div><Switch checked={autostart} onChange={setAutostart} /></div>
      </div>
      <div className="field-row">
        <div className="field-label">Run as user service</div>
        <div><Switch checked={true} /></div>
      </div>
      <FieldRow label="Socket path" help="Unix domain socket for IPC. Restart required if changed.">
        <input className="input" defaultValue="/run/user/1000/vibe-attack/vibe-attack.sock" />
      </FieldRow>
      <FieldRow label="Log level">
        <select className="select" defaultValue="info">
          <option>error</option>
          <option>warn</option>
          <option value="info">info</option>
          <option>debug</option>
          <option>trace</option>
        </select>
      </FieldRow>
    </Section>
    <Section title="Danger Zone">
      <div style={{ display: "flex", gap: 8 }}>
        <button className="btn btn-danger"><Icon name="trash" set={iconSet} size={12} />Reset to defaults</button>
        <button className="btn btn-danger">Wipe all profiles</button>
      </div>
    </Section>
  </>
);

window.ConfigApp = ConfigApp;
