# S02: AppImage complete build — UAT

**Milestone:** M005
**Written:** 2026-04-27T00:40:51.545Z

# S02 UAT — AppImage complete build\n\n- [ ] `bash packaging/appimage/build.sh` exits 0 on a machine with cargo, rsvg-convert, linuxdeploy, appimagetool\n- [ ] AppDir/usr/bin/ contains both vibe-attack and vibe-attack-config\n- [ ] AppDir/usr/lib/ contains libonnxruntime.so and libsherpa-onnx-c-api.so\n- [ ] AppRun contains LD_LIBRARY_PATH export covering usr/lib\n- [ ] Produced AppImage passes `--appimage-extract` sanity check
