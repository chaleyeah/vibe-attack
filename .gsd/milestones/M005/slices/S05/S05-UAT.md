# S05: RPM spec file — UAT

**Milestone:** M005
**Written:** 2026-04-27T00:41:16.453Z

# S05 UAT — RPM spec file\n\n- [ ] `rpmbuild -bs packaging/vibe-attack.spec` produces a .src.rpm without error\n- [ ] `rpmbuild -bb packaging/vibe-attack.spec` (with sources) produces .rpm with both binaries\n- [ ] `rpm -qpl vibe-attack-0.1.0-1.x86_64.rpm` lists /usr/bin/vibe-attack and /usr/bin/vibe-attack-config
