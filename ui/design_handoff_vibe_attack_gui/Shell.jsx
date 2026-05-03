// Shell.jsx — shared egui-faithful primitives
// Header, footer, left rail, status pill, LED meter, field row, etc.

const StatusPill = ({ state = "running" }) => {
  const labels = {
    running: "LISTENING",
    muted: "MUTED",
    error: "DAEMON ERROR",
    disconnected: "DAEMON OFFLINE",
  };
  return (
    <span className="status-pill" data-state={state}>
      <span className="dot"></span>
      {labels[state]}
    </span>
  );
};

const WmTitlebar = ({ title = "Vibe Attack" }) => (
  <div className="wm-titlebar">
    <span className="wm-icon">V</span>
    <span className="wm-title">{title}</span>
    <span className="wm-buttons">
      <span className="wm-btn"></span>
      <span className="wm-btn"></span>
      <span className="wm-btn"></span>
    </span>
  </div>
);

const AppHeader = ({ status = "running", iconSet = "svg", children }) => (
  <div className="app-header">
    <div className="app-brand">
      <span className="brand-mark">V</span>
      <span>VIBE ATTACK</span>
      <span className="brand-version">v0.7.2</span>
    </div>
    <StatusPill state={status} />
    <div className="header-spacer" />
    <div className="header-actions">
      {children}
      <button className="btn btn-ghost btn-icon" title="Diagnostics">
        <Icon name="diagnostics" set={iconSet} />
      </button>
      <button className="btn btn-ghost btn-icon" title="Refresh">
        <Icon name="refresh" set={iconSet} />
      </button>
    </div>
  </div>
);

// Left rail
const RAIL_ITEMS = [
  { id: "devices",   icon: "devices",   label: "Devices" },
  { id: "voice",     icon: "voice",     label: "Voice" },
  { id: "packs",     icon: "packs",     label: "Packs" },
  { id: "hotkeys",   icon: "hotkeys",   label: "Hotkeys" },
  { id: "advanced",  icon: "advanced",  label: "Advanced" },
];

const LeftRail = ({ active = "devices", onChange = () => {}, expanded = false, iconSet = "svg" }) => (
  <nav className="rail" data-expanded={expanded ? "true" : "false"}>
    {RAIL_ITEMS.map((it) => (
      <button
        key={it.id}
        className="rail-item"
        aria-current={active === it.id ? "true" : "false"}
        onClick={() => onChange(it.id)}
        title={it.label}
      >
        <span className="rail-icon">
          <Icon name={it.icon} set={iconSet} size={16} />
        </span>
        <span className="rail-label">{it.label}</span>
      </button>
    ))}
    <div className="rail-spacer" />
    <div className="rail-divider" />
    <button className="rail-item" title="Diagnostics">
      <span className="rail-icon"><Icon name="diagnostics" set={iconSet} size={16} /></span>
      <span className="rail-label">Diagnostics</span>
    </button>
  </nav>
);

// LED meter — `level` 0..1
const LedMeter = ({ level = 0, segments = 24 }) => {
  const cells = [];
  for (let i = 0; i < segments; i++) {
    const cellLevel = (i + 1) / segments;
    const on = level >= cellLevel - 0.001;
    let zone = "ok";
    if (cellLevel > 0.85) zone = "hot";
    else if (cellLevel > 0.65) zone = "warn";
    cells.push(
      <span key={i} className="led" data-on={on ? "true" : "false"} data-zone={zone}></span>
    );
  }
  return <div className="led-meter">{cells}</div>;
};

// Footer
const StatusFooter = ({ micLevel = 0.32, status = "running", model = "ggml-tiny.en" }) => {
  const stateText = {
    running: "ARMED",
    muted: "STANDBY",
    error: "FAULT",
    disconnected: "OFFLINE",
  }[status];
  return (
    <div className="app-footer">
      <div className="footer-cell" style={{ width: 200 }}>
        <span className="footer-label">MIC</span>
        <div style={{ flex: 1 }}>
          <LedMeter level={status === "muted" || status === "disconnected" ? 0 : micLevel} segments={20} />
        </div>
      </div>
      <div className="footer-divider" />
      <div className="footer-cell">
        <span className="footer-label">STATE</span>
        <span className="footer-value">{stateText}</span>
      </div>
      <div className="footer-divider" />
      <div className="footer-cell">
        <span className="footer-label">MODEL</span>
        <span className="footer-value">{model}</span>
      </div>
      <div className="footer-divider" />
      <div className="footer-cell">
        <span className="footer-label">SOCK</span>
        <span className="footer-value" style={{ color: status === "disconnected" || status === "error" ? "var(--err)" : "var(--ok)" }}>
          {status === "disconnected" || status === "error" ? "✕ unavailable" : "✓ /run/user/1000/vibe-attack/vibe-attack.sock"}
        </span>
      </div>
      <div style={{ flex: 1 }} />
      <div className="footer-cell">
        <span className="footer-label">UP</span>
        <span className="footer-value">{status === "disconnected" || status === "error" ? "—" : "00:42:18"}</span>
      </div>
    </div>
  );
};

// Field row
const FieldRow = ({ label, required, help, children }) => (
  <>
    <div className="field-row">
      <div className="field-label">
        {label}
        {required && <span className="req">*</span>}
      </div>
      <div>{children}</div>
    </div>
    {help && <div className="field-help">{help}</div>}
  </>
);

// Section
const Section = ({ title, sub, actions, children }) => (
  <section className="section">
    <div className="section-header">
      <div>
        <div className="section-title">{title}</div>
        {sub && <div className="section-sub" style={{ marginTop: 6 }}>{sub}</div>}
      </div>
      {actions && <div style={{ display: "flex", gap: 8 }}>{actions}</div>}
    </div>
    {children}
  </section>
);

// Slider
const Slider = ({ value = 0.5, min = 0, max = 1, suffix = "" }) => {
  const pct = ((value - min) / (max - min)) * 100;
  return (
    <div className="slider-row">
      <div className="slider-track">
        <div className="slider-fill" style={{ width: `${pct}%` }} />
        <div className="slider-thumb" style={{ left: `${pct}%` }} />
      </div>
      <div className="slider-value">{typeof value === "number" ? (max <= 1 ? Math.round(value * 100) : value) : value}{suffix}</div>
    </div>
  );
};

// Switch
const Switch = ({ checked = false, onChange = () => {} }) => (
  <label className="switch">
    <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
    <span className="track"><span className="thumb"></span></span>
  </label>
);

// Radio group
const RadioGroup = ({ value, options, onChange = () => {} }) => (
  <div className="radio-row">
    {options.map((opt) => (
      <label key={opt.value} className="radio">
        <input
          type="radio"
          checked={value === opt.value}
          onChange={() => onChange(opt.value)}
        />
        <span className="dot"></span>
        <span>{opt.label}</span>
      </label>
    ))}
  </div>
);

// Banner
const Banner = ({ kind = "info", title, children, actions }) => {
  const icons = { info: "i", warn: "!", error: "!", ok: "✓" };
  return (
    <div className="banner" data-kind={kind}>
      <span className="banner-icon">{icons[kind]}</span>
      <div className="banner-body">
        {title && <div className="banner-title">{title}</div>}
        <div className="banner-text">{children}</div>
        {actions && <div className="banner-actions">{actions}</div>}
      </div>
    </div>
  );
};

// Window shell — for screen-sized framings
const EguiWindow = ({ width = 1040, height = 720, title = "Vibe Attack Config", children, showWm = true }) => (
  <div
    className="egui-window"
    style={{ width, height }}
  >
    <span className="crosshair tl" />
    <span className="crosshair tr" />
    <span className="crosshair bl" />
    <span className="crosshair br" />
    {showWm && <WmTitlebar title={title} />}
    {children}
  </div>
);

Object.assign(window, {
  StatusPill, WmTitlebar, AppHeader, LeftRail, LedMeter, StatusFooter,
  FieldRow, Section, Slider, Switch, RadioGroup, Banner, EguiWindow,
  RAIL_ITEMS,
});
