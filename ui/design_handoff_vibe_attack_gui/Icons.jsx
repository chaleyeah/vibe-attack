// Icons.jsx — two icon sets, switchable by Tweak
// Set "svg":  carefully drawn line icons (1.5px stroke, 16x16 base) — would ship as PNG/SVG in assets/
// Set "unicode": egui's built-in symbols (no asset bundling)

const Icon = ({ name, size = 16, set = "svg", color = "currentColor" }) => {
  if (set === "unicode") {
    return <UnicodeIcon name={name} size={size} color={color} />;
  }
  return <SvgIcon name={name} size={size} color={color} />;
};

const UnicodeIcon = ({ name, size, color }) => {
  const map = {
    devices:   "▣",
    voice:     "◉",
    packs:     "▤",
    hotkeys:   "⌨",
    advanced:  "⚙",
    diagnostics: "◬",
    play:      "▶",
    stop:      "■",
    pause:     "❚❚",
    mic:       "◉",
    mute:      "⊘",
    error:     "⚠",
    info:      "ⓘ",
    check:     "✓",
    chevron:   "›",
    plus:      "+",
    minus:     "−",
    grip:      "⋮⋮",
    search:    "⌕",
    close:     "✕",
    menu:      "≡",
    refresh:   "↻",
    edit:      "✎",
    trash:     "🗑",
    folder:    "▤",
    download:  "↓",
    upload:    "↑",
    sound:     "♪",
    target:    "⊕",
    radio:     "◉",
    keyboard:  "⌨",
    save:      "💾",
    undo:      "↶",
    redo:      "↷",
  };
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        width: size,
        height: size,
        fontSize: Math.round(size * 0.95),
        lineHeight: 1,
        color,
        fontFamily: 'var(--font-mono)',
      }}
    >
      {map[name] || "•"}
    </span>
  );
};

const SvgIcon = ({ name, size, color }) => {
  const sw = 1.5;
  const props = {
    width: size,
    height: size,
    viewBox: "0 0 16 16",
    fill: "none",
    stroke: color,
    strokeWidth: sw,
    strokeLinecap: "round",
    strokeLinejoin: "round",
  };
  switch (name) {
    case "devices":
      return (
        <svg {...props}>
          <rect x="2" y="3" width="12" height="8" rx="1" />
          <path d="M5 13h6M8 11v2" />
        </svg>
      );
    case "voice":
      return (
        <svg {...props}>
          <rect x="6" y="2" width="4" height="8" rx="2" />
          <path d="M3.5 8a4.5 4.5 0 0 0 9 0M8 12.5V14M5.5 14h5" />
        </svg>
      );
    case "mic":
      return (
        <svg {...props}>
          <rect x="6" y="2" width="4" height="8" rx="2" />
          <path d="M3.5 8a4.5 4.5 0 0 0 9 0M8 12.5V14M5.5 14h5" />
        </svg>
      );
    case "mute":
      return (
        <svg {...props}>
          <rect x="6" y="2" width="4" height="8" rx="2" />
          <path d="M3.5 8a4.5 4.5 0 0 0 9 0M8 12.5V14M5.5 14h5" />
          <path d="M2 2l12 12" stroke={color} />
        </svg>
      );
    case "packs":
      return (
        <svg {...props}>
          <path d="M2 4l6-2 6 2v8l-6 2-6-2z" />
          <path d="M2 4l6 2 6-2M8 6v8" />
        </svg>
      );
    case "hotkeys":
    case "keyboard":
      return (
        <svg {...props}>
          <rect x="1.5" y="4" width="13" height="8" rx="1" />
          <path d="M4 7h.01M7 7h.01M10 7h.01M12.5 7h.01M4 9.5h8" />
        </svg>
      );
    case "advanced":
      return (
        <svg {...props}>
          <circle cx="8" cy="8" r="2.2" />
          <path d="M8 1.5v2M8 12.5v2M14.5 8h-2M3.5 8h-2M12.6 3.4l-1.4 1.4M4.8 11.2l-1.4 1.4M12.6 12.6l-1.4-1.4M4.8 4.8L3.4 3.4" />
        </svg>
      );
    case "diagnostics":
      return (
        <svg {...props}>
          <path d="M1.5 8h2.5l1.5-4 3 8 1.5-4h4" />
        </svg>
      );
    case "play":
      return (
        <svg {...props}>
          <path d="M4 3l8 5-8 5z" fill={color} stroke="none" />
        </svg>
      );
    case "stop":
      return (
        <svg {...props}>
          <rect x="4" y="4" width="8" height="8" fill={color} stroke="none" />
        </svg>
      );
    case "pause":
      return (
        <svg {...props}>
          <rect x="4" y="3" width="2.5" height="10" fill={color} stroke="none" />
          <rect x="9.5" y="3" width="2.5" height="10" fill={color} stroke="none" />
        </svg>
      );
    case "error":
      return (
        <svg {...props}>
          <circle cx="8" cy="8" r="6" />
          <path d="M8 5v3.5M8 10.5v.5" />
        </svg>
      );
    case "info":
      return (
        <svg {...props}>
          <circle cx="8" cy="8" r="6" />
          <path d="M8 7v4M8 5v.5" />
        </svg>
      );
    case "check":
      return (
        <svg {...props}>
          <path d="M3 8.5l3.5 3.5L13 5" />
        </svg>
      );
    case "chevron":
      return (
        <svg {...props}>
          <path d="M6 3.5l4.5 4.5L6 12.5" />
        </svg>
      );
    case "chevron-down":
      return (
        <svg {...props}>
          <path d="M3.5 6l4.5 4.5L12.5 6" />
        </svg>
      );
    case "chevron-left":
      return (
        <svg {...props}>
          <path d="M10 3.5L5.5 8 10 12.5" />
        </svg>
      );
    case "plus":
      return (
        <svg {...props}>
          <path d="M8 3v10M3 8h10" />
        </svg>
      );
    case "minus":
      return (
        <svg {...props}>
          <path d="M3 8h10" />
        </svg>
      );
    case "grip":
      return (
        <svg {...props} strokeWidth="0" fill={color}>
          <circle cx="6" cy="4" r="1" />
          <circle cx="10" cy="4" r="1" />
          <circle cx="6" cy="8" r="1" />
          <circle cx="10" cy="8" r="1" />
          <circle cx="6" cy="12" r="1" />
          <circle cx="10" cy="12" r="1" />
        </svg>
      );
    case "search":
      return (
        <svg {...props}>
          <circle cx="7" cy="7" r="4.5" />
          <path d="M10.5 10.5L13.5 13.5" />
        </svg>
      );
    case "close":
      return (
        <svg {...props}>
          <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" />
        </svg>
      );
    case "refresh":
      return (
        <svg {...props}>
          <path d="M13.5 4.5A6 6 0 1 0 14 9" />
          <path d="M14 2v3h-3" />
        </svg>
      );
    case "edit":
      return (
        <svg {...props}>
          <path d="M11 2.5l2.5 2.5L5 13.5H2.5V11z" />
        </svg>
      );
    case "trash":
      return (
        <svg {...props}>
          <path d="M2.5 4h11M5 4V2.5h6V4M4 4l1 9.5h6L12 4M6.5 6.5v5M9.5 6.5v5" />
        </svg>
      );
    case "folder":
      return (
        <svg {...props}>
          <path d="M1.5 4.5h4l1.5 1.5h7.5v7h-13z" />
        </svg>
      );
    case "download":
      return (
        <svg {...props}>
          <path d="M8 2v9M4 7l4 4 4-4M2.5 13.5h11" />
        </svg>
      );
    case "upload":
      return (
        <svg {...props}>
          <path d="M8 11V2M4 6l4-4 4 4M2.5 13.5h11" />
        </svg>
      );
    case "target":
      return (
        <svg {...props}>
          <circle cx="8" cy="8" r="6" />
          <circle cx="8" cy="8" r="3" />
          <circle cx="8" cy="8" r="0.5" fill={color} />
        </svg>
      );
    case "radio":
      return (
        <svg {...props}>
          <circle cx="8" cy="8" r="6" />
          <circle cx="8" cy="8" r="2.5" fill={color} stroke="none" />
        </svg>
      );
    case "save":
      return (
        <svg {...props}>
          <path d="M2.5 2.5h9L13.5 4.5v9h-11z" />
          <path d="M5 2.5v3.5h6V2.5M5 9h6v4.5H5z" />
        </svg>
      );
    case "undo":
      return (
        <svg {...props}>
          <path d="M3 7h7a3.5 3.5 0 0 1 0 7H7" />
          <path d="M5.5 4.5L3 7l2.5 2.5" />
        </svg>
      );
    case "menu":
      return (
        <svg {...props}>
          <path d="M2.5 4.5h11M2.5 8h11M2.5 11.5h11" />
        </svg>
      );
    case "shield":
      return (
        <svg {...props}>
          <path d="M8 1.5L13.5 3v5c0 3-2.5 5.5-5.5 6.5C5 13.5 2.5 11 2.5 8V3z" />
        </svg>
      );
    case "wave":
      return (
        <svg {...props}>
          <path d="M2 8h1.5L5 4l2 8 2-6 2 4h3" />
        </svg>
      );
    default:
      return (
        <svg {...props}>
          <circle cx="8" cy="8" r="2" fill={color} stroke="none" />
        </svg>
      );
  }
};

window.Icon = Icon;
