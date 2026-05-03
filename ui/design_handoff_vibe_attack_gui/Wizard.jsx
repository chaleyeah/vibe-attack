// Wizard.jsx — first-run setup flow

const WIZARD_STEPS = [
  { id: "welcome", label: "Welcome" },
  { id: "audio",   label: "Audio" },
  { id: "test",    label: "Mic test" },
  { id: "ptt",     label: "PTT key" },
  { id: "model",   label: "Model" },
  { id: "done",    label: "Finish" },
];

const Wizard = ({ initialStep = 2, iconSet = "svg" }) => {
  const [step, setStep] = React.useState(initialStep);
  const [device, setDevice] = React.useState("yeti");
  const [bindKey, setBindKey] = React.useState(null);
  const [listening, setListening] = React.useState(false);
  const [model, setModel] = React.useState("tiny");

  const [micLvl, setMicLvl] = React.useState(0.4);
  React.useEffect(() => {
    const t = setInterval(() => setMicLvl(0.2 + Math.random() * 0.6), 180);
    return () => clearInterval(t);
  }, []);

  const next = () => setStep((s) => Math.min(WIZARD_STEPS.length - 1, s + 1));
  const back = () => setStep((s) => Math.max(0, s - 1));

  return (
    <>
      <AppHeader status="running" iconSet={iconSet} />

      <div style={{ flex: 1, display: "flex", flexDirection: "column", background: "var(--bg-window)", overflow: "hidden" }}>
        {/* Step indicator */}
        <div style={{ padding: "20px 32px 16px", borderBottom: "1px solid var(--stroke-faint)" }}>
          <div style={{ fontSize: 10, letterSpacing: "0.18em", color: "var(--fg-faint)", textTransform: "uppercase", marginBottom: 12 }}>
            FIRST-RUN SETUP · STEP {step + 1} OF {WIZARD_STEPS.length}
          </div>
          <div className="steps">
            {WIZARD_STEPS.map((s, i) => (
              <React.Fragment key={s.id}>
                <div className="step" data-state={i < step ? "done" : i === step ? "active" : "pending"}>
                  <span className="step-num">{i < step ? "✓" : i + 1}</span>
                  <span className="step-label">{s.label}</span>
                </div>
                {i < WIZARD_STEPS.length - 1 && <div className="step-link" />}
              </React.Fragment>
            ))}
          </div>
        </div>

        {/* Body */}
        <div style={{ flex: 1, overflowY: "auto", padding: "32px 48px" }}>
          {step === 0 && <StepWelcome iconSet={iconSet} />}
          {step === 1 && <StepAudio device={device} setDevice={setDevice} iconSet={iconSet} />}
          {step === 2 && <StepMicTest micLvl={micLvl} iconSet={iconSet} />}
          {step === 3 && <StepPtt bindKey={bindKey} setBindKey={setBindKey} listening={listening} setListening={setListening} iconSet={iconSet} />}
          {step === 4 && <StepModel model={model} setModel={setModel} iconSet={iconSet} />}
          {step === 5 && <StepDone iconSet={iconSet} />}
        </div>

        {/* Footer nav */}
        <div style={{ height: 60, borderTop: "1px solid var(--stroke-faint)", background: "var(--bg-panel)", display: "flex", alignItems: "center", padding: "0 24px", gap: 8 }}>
          <button className="btn btn-ghost" onClick={back} disabled={step === 0}>
            <Icon name="chevron-left" set={iconSet} size={12} />Back
          </button>
          <div style={{ flex: 1 }} />
          {step > 0 && step < WIZARD_STEPS.length - 1 && (
            <button className="btn btn-ghost" onClick={next}>Skip for now</button>
          )}
          {step < WIZARD_STEPS.length - 1 ? (
            <button className="btn btn-primary" onClick={next}>
              Continue
              <Icon name="chevron" set={iconSet} size={12} />
            </button>
          ) : (
            <button className="btn btn-primary">
              <Icon name="check" set={iconSet} size={12} />Launch daemon
            </button>
          )}
        </div>
      </div>
    </>
  );
};

const StepWelcome = ({ iconSet }) => (
  <div style={{ maxWidth: 540, margin: "20px auto 0" }}>
    <div style={{ fontSize: 11, letterSpacing: "0.2em", color: "var(--accent)", textTransform: "uppercase", marginBottom: 14 }}>
      ▸ INITIALIZE
    </div>
    <h2 style={{ fontSize: 24, color: "var(--fg-strong)", margin: "0 0 12px", fontWeight: 500 }}>Configure your voice macros</h2>
    <p style={{ color: "var(--fg-muted)", fontSize: 13, lineHeight: 1.7, margin: "0 0 28px", maxWidth: 480 }}>
      Vibe Attack listens for spoken phrases and emulates keyboard input. Five quick steps will get you armed: pick your mic, run a level test, bind a push-to-talk key, choose a model, and you're done.
    </p>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
      {[
        { k: "AUDIO", v: "Capture device + sample rate" },
        { k: "VAD",   v: "Voice activity detection" },
        { k: "INPUT", v: "PTT key via uinput / evdev" },
        { k: "STT",   v: "Local whisper.cpp model" },
      ].map((c) => (
        <div key={c.k} style={{ padding: "12px 14px", background: "var(--bg-faint)", border: "1px solid var(--stroke)", borderRadius: 4 }}>
          <div style={{ fontSize: 10, letterSpacing: "0.16em", color: "var(--accent)" }}>{c.k}</div>
          <div style={{ fontSize: 12, color: "var(--fg)", marginTop: 4 }}>{c.v}</div>
        </div>
      ))}
    </div>
  </div>
);

const StepAudio = ({ device, setDevice, iconSet }) => (
  <div style={{ maxWidth: 580, margin: "0 auto" }}>
    <h3 style={{ fontSize: 18, color: "var(--fg-strong)", margin: "0 0 6px", fontWeight: 500 }}>Pick a capture device</h3>
    <p style={{ color: "var(--fg-muted)", fontSize: 12, marginBottom: 24 }}>The daemon will hold this device exclusively while running.</p>

    <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
      {[
        { id: "yeti",   name: "Blue Yeti — USB Audio", meta: "alsa_input.usb-Blue_Yeti", rate: "48 kHz · 2ch", recommended: true },
        { id: "hdmic",  name: "Built-in Microphone", meta: "alsa_input.pci-0000_00_1f.3", rate: "44.1 kHz · 1ch" },
        { id: "bt",     name: "Sony WH-1000XM4", meta: "bluez_input.AC_80_0A_xx.handsfree", rate: "16 kHz · 1ch", warn: "Bluetooth handsfree drops sample rate" },
      ].map((d) => (
        <button
          key={d.id}
          className="list-row"
          onClick={() => setDevice(d.id)}
          aria-selected={device === d.id ? "true" : "false"}
          style={{ padding: "12px 14px", background: device === d.id ? "var(--accent-faint)" : "var(--bg-faint)", borderColor: device === d.id ? "var(--accent-line)" : "var(--stroke)" }}
        >
          <Icon name={device === d.id ? "radio" : "target"} set={iconSet} color={device === d.id ? "var(--accent)" : "var(--fg-faint)"} />
          <div style={{ flex: 1, textAlign: "left" }}>
            <div style={{ fontSize: 13, color: "var(--fg-strong)", display: "flex", gap: 8, alignItems: "center" }}>
              {d.name}
              {d.recommended && <span className="tag" data-tone="ok">RECOMMENDED</span>}
            </div>
            <div style={{ fontSize: 10, color: "var(--fg-faint)", letterSpacing: "0.06em", marginTop: 4 }}>
              {d.meta} · {d.rate}
            </div>
            {d.warn && <div style={{ fontSize: 11, color: "var(--warn)", marginTop: 4 }}>⚠ {d.warn}</div>}
          </div>
        </button>
      ))}
    </div>
  </div>
);

const StepMicTest = ({ micLvl, iconSet }) => (
  <div style={{ maxWidth: 580, margin: "0 auto" }}>
    <h3 style={{ fontSize: 18, color: "var(--fg-strong)", margin: "0 0 6px", fontWeight: 500 }}>Speak normally</h3>
    <p style={{ color: "var(--fg-muted)", fontSize: 12, marginBottom: 28 }}>Aim for the green band when speaking at game volume. Adjust your OS input gain if you can't reach it.</p>

    <div style={{ background: "var(--bg-extreme)", border: "1px solid var(--stroke)", borderRadius: 4, padding: "20px 24px" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 14 }}>
        <Icon name="mic" set={iconSet} color="var(--accent)" />
        <span style={{ fontSize: 11, letterSpacing: "0.14em", color: "var(--fg)" }}>BLUE YETI — USB AUDIO</span>
        <div style={{ flex: 1 }} />
        <span style={{ fontSize: 11, color: micLvl > 0.85 ? "var(--err)" : micLvl > 0.65 ? "var(--warn)" : "var(--ok)", fontVariantNumeric: "tabular-nums" }}>
          {micLvl > 0.85 ? "TOO HOT" : micLvl > 0.65 ? "GOOD" : micLvl > 0.2 ? "OK" : "QUIET"} · {Math.round(micLvl * 100)}%
        </span>
      </div>

      <LedMeter level={micLvl} segments={40} />

      <div style={{ display: "flex", justifyContent: "space-between", marginTop: 8, fontSize: 9, color: "var(--fg-faint)", letterSpacing: "0.1em" }}>
        <span>FLOOR</span>
        <span>NOISE</span>
        <span>SPEECH</span>
        <span>HOT</span>
        <span>CLIP</span>
      </div>
    </div>

    <div style={{ marginTop: 20 }}>
      <Banner kind="info" title="DETECTED">
        VAD captured 3 utterances in the last 5 seconds. Average confidence <code>0.91</code>. Looks healthy.
      </Banner>
    </div>
  </div>
);

const StepPtt = ({ bindKey, setBindKey, listening, setListening, iconSet }) => (
  <div style={{ maxWidth: 540, margin: "0 auto" }}>
    <h3 style={{ fontSize: 18, color: "var(--fg-strong)", margin: "0 0 6px", fontWeight: 500 }}>Bind your push-to-talk key</h3>
    <p style={{ color: "var(--fg-muted)", fontSize: 12, marginBottom: 24 }}>Hold to capture audio. Skip this step to use wake-word mode instead.</p>

    <div
      style={{
        padding: "40px 24px",
        background: listening ? "var(--accent-faint)" : "var(--bg-faint)",
        border: `1px ${listening ? "solid" : "dashed"} ${listening ? "var(--accent-line)" : "var(--stroke-strong)"}`,
        borderRadius: 4,
        textAlign: "center",
      }}
    >
      {listening ? (
        <>
          <div style={{ fontSize: 11, letterSpacing: "0.18em", color: "var(--accent)", marginBottom: 14 }}>
            ▸ LISTENING — PRESS A KEY
          </div>
          <div style={{ display: "flex", justifyContent: "center", gap: 4 }}>
            {[0, 1, 2].map((i) => (
              <span key={i} style={{ width: 6, height: 6, borderRadius: 50, background: "var(--accent)", animation: `pulse 1s ease-in-out ${i * 0.15}s infinite` }} />
            ))}
          </div>
        </>
      ) : (
        <>
          <div style={{ marginBottom: 14 }}>
            {bindKey ? (
              <span className="kbd" style={{ height: 36, fontSize: 14, padding: "0 14px" }}>{bindKey}</span>
            ) : (
              <span className="kbd" style={{ height: 36, fontSize: 14, padding: "0 14px", color: "var(--fg-faint)" }}>Left Ctrl</span>
            )}
          </div>
          <button className="btn" onClick={() => { setListening(true); setTimeout(() => { setBindKey("F12"); setListening(false); }, 1200); }}>
            <Icon name="keyboard" set={iconSet} size={12} />Capture new binding
          </button>
        </>
      )}
    </div>

    <div style={{ marginTop: 20, fontSize: 11, color: "var(--fg-muted)", lineHeight: 1.6 }}>
      <div style={{ letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--fg-faint)", marginBottom: 6 }}>NOTES</div>
      <ul style={{ margin: 0, paddingLeft: 18 }}>
        <li>Daemon reads via <code style={{ background: "var(--bg-extreme)", padding: "1px 4px", border: "1px solid var(--stroke)" }}>/dev/input/event*</code> — your user must be in the <code style={{ background: "var(--bg-extreme)", padding: "1px 4px", border: "1px solid var(--stroke)" }}>input</code> group.</li>
        <li>Modifier-only bindings work (Ctrl, Alt, Super). Combos are bound on key-up.</li>
      </ul>
    </div>
  </div>
);

const StepModel = ({ model, setModel, iconSet }) => (
  <div style={{ maxWidth: 580, margin: "0 auto" }}>
    <h3 style={{ fontSize: 18, color: "var(--fg-strong)", margin: "0 0 6px", fontWeight: 500 }}>Choose a Whisper model</h3>
    <p style={{ color: "var(--fg-muted)", fontSize: 12, marginBottom: 24 }}>All models run locally via whisper.cpp. No audio leaves your machine.</p>

    <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
      {[
        { id: "tiny",   name: "ggml-tiny.en",  size: "75 MB",  speed: "≈ 80 ms",  acc: "BASIC",  rec: true },
        { id: "base",   name: "ggml-base.en",  size: "142 MB", speed: "≈ 130 ms", acc: "GOOD" },
        { id: "small",  name: "ggml-small.en", size: "466 MB", speed: "≈ 320 ms", acc: "GREAT" },
        { id: "medium", name: "ggml-medium.en",size: "1.5 GB", speed: "≈ 950 ms", acc: "BEST", warn: "Heavy on CPU mid-game" },
      ].map((m) => (
        <button
          key={m.id}
          className="list-row"
          onClick={() => setModel(m.id)}
          aria-selected={model === m.id ? "true" : "false"}
          style={{ padding: "12px 14px", background: model === m.id ? "var(--accent-faint)" : "var(--bg-faint)", borderColor: model === m.id ? "var(--accent-line)" : "var(--stroke)" }}
        >
          <Icon name={model === m.id ? "radio" : "target"} set={iconSet} color={model === m.id ? "var(--accent)" : "var(--fg-faint)"} />
          <div style={{ flex: 1, textAlign: "left" }}>
            <div style={{ fontSize: 13, color: "var(--fg-strong)", display: "flex", gap: 8, alignItems: "center" }}>
              {m.name}
              {m.rec && <span className="tag" data-tone="ok">RECOMMENDED FOR GAMING</span>}
            </div>
            <div style={{ fontSize: 10, color: "var(--fg-faint)", letterSpacing: "0.06em", marginTop: 4 }}>
              {m.size} · LATENCY {m.speed} · ACCURACY {m.acc}
            </div>
            {m.warn && <div style={{ fontSize: 11, color: "var(--warn)", marginTop: 4 }}>⚠ {m.warn}</div>}
          </div>
          <span className="tag">DOWNLOAD</span>
        </button>
      ))}
    </div>
  </div>
);

const StepDone = ({ iconSet }) => (
  <div style={{ maxWidth: 480, margin: "20px auto 0", textAlign: "center" }}>
    <div style={{ width: 56, height: 56, margin: "0 auto 20px", border: "1px solid var(--ok)", borderRadius: 50, display: "flex", alignItems: "center", justifyContent: "center", background: "var(--ok-faint)" }}>
      <Icon name="check" set={iconSet} size={28} color="var(--ok)" />
    </div>
    <div style={{ fontSize: 11, letterSpacing: "0.2em", color: "var(--ok)", marginBottom: 14 }}>▸ READY</div>
    <h3 style={{ fontSize: 22, color: "var(--fg-strong)", fontWeight: 500, margin: "0 0 12px" }}>You're armed and weaponized</h3>
    <p style={{ color: "var(--fg-muted)", fontSize: 13, lineHeight: 1.7 }}>
      The daemon will start at login and stay in your tray. Drop into Helldivers 2 and try saying the name of any stratagem — the macro will fire if it's in your active pack.
    </p>
  </div>
);

window.Wizard = Wizard;
