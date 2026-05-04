Name:           vibe-attack
Version:        1.0.0
Release:        1%{?dist}
Summary:        Voice macro daemon for Helldivers 2 on Linux

License:        AGPL-3.0-only
URL:            https://github.com/chaleyeah/vibe-attack
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

ExclusiveArch:  x86_64

BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  clang-devel
BuildRequires:  alsa-lib-devel
BuildRequires:  patchelf

Requires:       alsa-lib

# Bundled libs (not in Fedora/RHEL repos): sherpa-onnx + onnxruntime
# Suppress auto-provides for bundled libs to avoid polluting system
%global __provides_exclude_from ^%{_libdir}/%{name}/.*\\.so.*$
%global __requires_exclude ^libsherpa-onnx-.*\\.so.*$|^libonnxruntime\\.so.*$

%description
Vibe Attack listens for voice commands and injects keyboard macros,
allowing hands-free execution of stratagems and other in-game actions
in Helldivers 2 on Linux.

Includes vibe-attack (daemon) and vibe-attack-config (GUI configuration tool).

%prep
%autosetup

%build
cargo build --release --locked
cargo build --release --locked --features gui

%install
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dm755 target/release/%{name}-config %{buildroot}%{_bindir}/%{name}-config
install -Dm644 packaging/appimage/vibe-attack.desktop \
    %{buildroot}%{_datadir}/applications/vibe-attack.desktop
install -Dm644 assets/vibe-attack.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/vibe-attack.svg

# Bundle sherpa-onnx + onnxruntime shared libs into private libdir
install -d %{buildroot}%{_libdir}/%{name}
install -m644 target/release/libsherpa-onnx-c-api.so %{buildroot}%{_libdir}/%{name}/
install -m644 target/release/libsherpa-onnx-cxx-api.so %{buildroot}%{_libdir}/%{name}/
install -m644 target/release/libonnxruntime.so %{buildroot}%{_libdir}/%{name}/

# Set rpath so binaries find bundled libs
patchelf --set-rpath '$ORIGIN/../%{_lib}/%{name}' %{buildroot}%{_bindir}/%{name}
patchelf --set-rpath '$ORIGIN/../%{_lib}/%{name}' %{buildroot}%{_bindir}/%{name}-config

# Install Silero VAD ONNX model (baked path in crate points to build machine's cargo registry)
SILERO_MODEL=$(find "${CARGO_HOME:-${HOME}/.cargo}/registry/src" -name 'silero_vad.onnx' -path '*silero-vad-rust*' 2>/dev/null | head -1)
install -Dm644 "$SILERO_MODEL" %{buildroot}%{_datadir}/%{name}/silero_vad.onnx

%check
# Audio hardware not available in build env — skip runtime tests

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_bindir}/%{name}-config
%{_libdir}/%{name}/
%{_datadir}/%{name}/
%{_datadir}/applications/vibe-attack.desktop
%{_datadir}/icons/hicolor/scalable/apps/vibe-attack.svg

%changelog
* Tue Apr 28 2026 Chris Chale <chrischale@gmail.com> - 1.0.0-1
- Version 1.0.0 release

* Sun Apr 26 2026 Chris Chale <chrischale@gmail.com> - 0.1.0-1
- Initial packaging
