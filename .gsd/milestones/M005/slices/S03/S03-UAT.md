# S03: Debian package — UAT

**Milestone:** M005
**Written:** 2026-04-27T00:40:58.954Z

# S03 UAT — Debian package\n\n- [ ] `dpkg-buildpackage -us -uc -b` in project root produces vibe-attack_0.1.0-1_amd64.deb\n- [ ] `dpkg --info vibe-attack_0.1.0-1_amd64.deb` shows correct Depends, Architecture: amd64\n- [ ] `dpkg -i vibe-attack_0.1.0-1_amd64.deb` installs both binaries to /usr/bin/\n- [ ] Icon present at /usr/share/icons/hicolor/scalable/apps/vibe-attack.svg after install
