// PackEditor.jsx — two-pane voice pack editor

const PackEditor = ({ iconSet = "svg" }) => {
  const [profile] = React.useState("Stratagems");
  const [search, setSearch] = React.useState("");
  const [selectedCat, setSelectedCat] = React.useState("offensive");
  const [selectedMacro, setSelectedMacro] = React.useState("eagle-airstrike");
  const [drag, setDrag] = React.useState(null);

  const categories = [
    { id: "offensive", name: "Offensive", count: 8 },
    { id: "defensive", name: "Defensive", count: 6 },
    { id: "supply",    name: "Supply",    count: 5 },
    { id: "mission",   name: "Mission",   count: 3 },
    { id: "comms",     name: "Squad comms", count: 2 },
  ];

  const macros = {
    offensive: [
      { id: "eagle-airstrike",  name: "Eagle Airstrike",      phrase: "eagle airstrike",      keys: "↓ → ↓ →",          flag: "stratagem.aerial" },
      { id: "orbital-precision", name: "Orbital Precision",   phrase: "precision strike",     keys: "→ → ↑",            flag: "stratagem.orbital" },
      { id: "eagle-cluster",    name: "Eagle Cluster",        phrase: "cluster bomb",         keys: "↑ → ↓ ↓ →",        flag: "stratagem.aerial" },
      { id: "orbital-laser",    name: "Orbital Laser",        phrase: "laser",                keys: "→ ↓ ↑ → ↓",        flag: "stratagem.orbital" },
      { id: "500kg",            name: "500kg Bomb",           phrase: "five hundred",         keys: "↑ → ↓ ↓ ↓",        flag: "stratagem.aerial" },
      { id: "railcannon",       name: "Railcannon Strike",    phrase: "railcannon",           keys: "→ ↑ ↓ ↓ →",        flag: "stratagem.orbital" },
      { id: "napalm",           name: "Eagle Napalm",         phrase: "napalm",               keys: "→ ↓ ↑",            flag: "stratagem.aerial" },
      { id: "gas-strike",       name: "Orbital Gas",          phrase: "gas strike",           keys: "→ → ↓ →",          flag: "stratagem.orbital" },
    ],
  };

  const list = (macros[selectedCat] || []).filter((m) =>
    !search || m.name.toLowerCase().includes(search.toLowerCase()) || m.phrase.includes(search.toLowerCase())
  );
  const macro = list.find((m) => m.id === selectedMacro) || list[0];

  return (
    <>
      <AppHeader status="running" iconSet={iconSet}>
        <button className="btn btn-ghost"><Icon name="undo" set={iconSet} size={12} />Undo</button>
        <button className="btn"><Icon name="download" set={iconSet} size={12} />Export</button>
        <button className="btn btn-primary"><Icon name="save" set={iconSet} size={12} />Save pack</button>
      </AppHeader>

      <div className="app-body">
        <LeftRail active="packs" iconSet={iconSet} />

        <div style={{ flex: 1, display: "flex", flexDirection: "column", minWidth: 0 }}>
          {/* Subheader — pack identity */}
          <div style={{ padding: "14px 24px", background: "var(--bg-panel)", borderBottom: "1px solid var(--stroke-faint)", display: "flex", alignItems: "center", gap: 14 }}>
            <Icon name="packs" set={iconSet} color="var(--accent)" size={20} />
            <div>
              <div style={{ fontSize: 10, letterSpacing: "0.16em", color: "var(--fg-faint)", textTransform: "uppercase" }}>EDITING PACK</div>
              <div style={{ fontSize: 16, color: "var(--fg-strong)", marginTop: 2 }}>{profile}</div>
            </div>
            <span className="tag" data-tone="accent">ACTIVE</span>
            <span className="tag">24 MACROS</span>
            <div style={{ flex: 1 }} />
            <span style={{ fontSize: 11, color: "var(--fg-muted)" }}>~/.config/vibe-attack/packs/stratagems.toml</span>
          </div>

          {/* Three-pane: categories | macros | detail */}
          <div style={{ flex: 1, display: "grid", gridTemplateColumns: "200px 280px 1fr", minHeight: 0 }}>
            {/* Categories */}
            <div style={{ borderRight: "1px solid var(--stroke-faint)", background: "var(--bg-panel)", display: "flex", flexDirection: "column" }}>
              <div style={{ padding: "12px 14px 8px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <div style={{ fontSize: 10, letterSpacing: "0.16em", color: "var(--fg-faint)" }}>CATEGORIES</div>
                <button className="btn btn-ghost btn-icon" title="Add category"><Icon name="plus" set={iconSet} size={12} /></button>
              </div>
              <div style={{ flex: 1, overflowY: "auto", padding: "0 8px 8px" }}>
                {categories.map((c) => (
                  <button
                    key={c.id}
                    className="list-row"
                    aria-selected={selectedCat === c.id ? "true" : "false"}
                    onClick={() => setSelectedCat(c.id)}
                  >
                    <span className="grip"><Icon name="grip" set={iconSet} size={12} /></span>
                    <span className="row-name">{c.name}</span>
                    <span className="row-meta">{c.count}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Macros */}
            <div style={{ borderRight: "1px solid var(--stroke-faint)", display: "flex", flexDirection: "column", background: "var(--bg-window)" }}>
              <div style={{ padding: "10px 12px", borderBottom: "1px solid var(--stroke-faint)" }}>
                <div style={{ position: "relative" }}>
                  <span style={{ position: "absolute", left: 10, top: "50%", transform: "translateY(-50%)", color: "var(--fg-faint)" }}>
                    <Icon name="search" set={iconSet} size={14} />
                  </span>
                  <input
                    className="input"
                    placeholder="Filter macros…"
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                    style={{ paddingLeft: 32 }}
                  />
                </div>
              </div>
              <div style={{ flex: 1, overflowY: "auto", padding: 8 }}>
                {list.map((m, i) => (
                  <div
                    key={m.id}
                    className="list-row"
                    aria-selected={macro?.id === m.id ? "true" : "false"}
                    onClick={() => setSelectedMacro(m.id)}
                    onDragStart={() => setDrag(i)}
                    draggable
                    style={{ padding: "8px 10px", flexDirection: "column", alignItems: "stretch", gap: 4 }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                      <span className="grip"><Icon name="grip" set={iconSet} size={12} /></span>
                      <span style={{ flex: 1, fontSize: 12, color: "var(--fg-strong)" }}>{m.name}</span>
                      <span style={{ fontSize: 10, color: "var(--fg-faint)", fontVariantNumeric: "tabular-nums" }}>{String(i + 1).padStart(2, "0")}</span>
                    </div>
                    <div style={{ fontSize: 10, color: "var(--fg-faint)", letterSpacing: "0.04em", marginLeft: 22 }}>
                      "{m.phrase}"
                    </div>
                  </div>
                ))}
              </div>
              <div style={{ padding: 8, borderTop: "1px solid var(--stroke-faint)" }}>
                <button className="btn" style={{ width: "100%" }}>
                  <Icon name="plus" set={iconSet} size={12} />Add macro
                </button>
              </div>
            </div>

            {/* Detail */}
            <div style={{ overflowY: "auto", background: "var(--bg-window)" }}>
              {macro ? <MacroDetail macro={macro} iconSet={iconSet} /> : (
                <div style={{ padding: 40, textAlign: "center", color: "var(--fg-faint)" }}>Select a macro to edit</div>
              )}
            </div>
          </div>
        </div>
      </div>

      <StatusFooter status="running" micLevel={0.4} />
    </>
  );
};

const MacroDetail = ({ macro, iconSet }) => (
  <>
    <Section title={`Macro · ${macro.name}`} sub="Triggered by exact phrase match (case-insensitive)" actions={
      <>
        <button className="btn btn-ghost"><Icon name="play" set={iconSet} size={12} />Test</button>
        <button className="btn btn-danger btn-icon" title="Delete"><Icon name="trash" set={iconSet} size={12} /></button>
      </>
    }>
      <FieldRow label="Name" required>
        <input className="input" defaultValue={macro.name} />
      </FieldRow>

      <FieldRow label="Trigger phrase" required help="What you say. Variants separated by | — e.g. eagle airstrike | airstrike">
        <input className="input" defaultValue={macro.phrase} />
      </FieldRow>

      <FieldRow label="Min confidence" help="Override pack default. Empty inherits from voice settings.">
        <Slider value={0.78} />
      </FieldRow>
    </Section>

    <Section title="Key Sequence" sub="Emitted via uinput when phrase matches">
      <div style={{ background: "var(--bg-extreme)", border: "1px solid var(--stroke)", borderRadius: 4, padding: 14, marginBottom: 12 }}>
        <div style={{ display: "flex", gap: 6, alignItems: "center", flexWrap: "wrap" }}>
          <span className="kbd">Ctrl</span>
          <span style={{ color: "var(--fg-faint)" }}>+</span>
          {macro.keys.split(" ").map((k, i) => (
            <React.Fragment key={i}>
              <span className="kbd">{k}</span>
              {i < macro.keys.split(" ").length - 1 && <span style={{ color: "var(--fg-faint)" }}>→</span>}
            </React.Fragment>
          ))}
          <div style={{ flex: 1 }} />
          <button className="btn btn-ghost"><Icon name="edit" set={iconSet} size={12} />Capture</button>
        </div>
      </div>

      <FieldRow label="Hold time" help="Per-key duration in milliseconds.">
        <Slider value={50} min={10} max={250} suffix=" ms" />
      </FieldRow>

      <FieldRow label="Inter-key delay">
        <Slider value={75} min={0} max={500} suffix=" ms" />
      </FieldRow>
    </Section>

    <Section title="Conditions" sub="Optional flag gating (set by other macros or manually)">
      <FieldRow label="Required flag" help="Macro fires only if this flag is currently set.">
        <input className="input" defaultValue={macro.flag} placeholder="(none)" />
      </FieldRow>
      <FieldRow label="Set flag on fire" help="Sets a flag after firing — useful for chained sequences.">
        <input className="input" placeholder="(none)" />
      </FieldRow>
      <FieldRow label="Cooldown">
        <Slider value={500} min={0} max={5000} suffix=" ms" />
      </FieldRow>
    </Section>
  </>
);

window.PackEditor = PackEditor;
