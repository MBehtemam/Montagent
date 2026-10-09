---
status: accepted
amends: 0115 (§7: Windows no longer installs the newest release through `choco`; both Windows legs install a pinned, checksum-verified BtbN build of the newest release series, and macOS alone floats)
---

# Windows CI pins a checksummed ffmpeg, and macOS alone floats

[#866](https://github.com/MBehtemam/Montagent/pull/866).
[ADR-0115](0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md)
§7 has macOS (`brew`) and Windows (`choco`) install the newest release, so that the next ffmpeg
that removes an option Montagent uses turns CI red without anyone bumping a pin. On Windows
that floating install failed twice in ways unrelated to Montagent's arguments.

## Why Windows stopped floating

Both measurements are from #866's description. No committed script re-derives them.

- **x64: the floating build was not deterministic.** `choco`'s `ffmpeg` is gyan.dev's
  `9.0.2-essentials_build`. In CI run 37908861657, 9 of `painters.rs`'s 25 tests failed, and
  the run before it passed 25 of 25. In every failure the only field that differed was the
  encoded file's `bytes`, in both directions (for example 18992 against 19012). The per-frame
  hashes at the encoder input were identical, and the thread count was ADR-0143's pinned 5.
- **arm64: no usable native build floats.** `choco` installs gyan.dev's x64 build on every
  machine, and gyan.dev has no arm64 build. So on `windows-11-arm` every encode, decode and
  probe ran under emulation, about 1.5× slower than Linux outside Skia. That was enough to miss
  `preview_budget`'s scrub budget (5.0 s against 5.0 s). BtbN is the only source of a native
  arm64 build, and its newest one was unusable. The winarm64 build in
  `autobuild-2026-09-30-13-08` crashed on `ffmpeg -version` with 0xC0000005. BtbN's builds from
  2026-09-23 to 2026-10-03 ran `llvm-strip --strip-unneeded`, which made lld skip relocations
  in winarm64's llvm-mingw link. #866 counted 162 unrelocated calls in that `ffmpeg.exe` and 0
  in the 2026-08-31 build.

## The decision

**Both Windows legs install a pinned BtbN GPL static build of the newest release series,
verified by SHA-256.** `ci/install_ffmpeg_windows.ps1` holds the pins:

| Leg | Build | Release |
| --- | --- | --- |
| x64 (`win64`) | `ffmpeg-n9.0.2-17-g2a571b6068` | `autobuild-2026-09-30-13-08` |
| arm64 (`winarm64`) | `ffmpeg-n9.0.1-11-ge47273f4d9` | `autobuild-2026-08-31-13-27` |

The checksums come from BtbN's `checksums.sha256` for each release. The pins use month-end
releases for the reason ADR-0115 §7 gives for the floor: BtbN prunes dailies but keeps
month-end releases. This is the same builder the Linux floor already uses. If the arm64 build
does not run, the script warns and installs the x64 pin to run under emulation.

**macOS still floats.** It is now the only leg that installs the newest release without a pin,
and the only one that catches a new ffmpeg that removes an option. ADR-0115's purpose for "the
top" holds through it, because the arguments are the same on every OS.

**Maintenance cost.** A pin changes only when someone bumps it. Bump the Windows pins when
BtbN ships a month-end release of a newer build or series (9.1, 10), so that Windows keeps
testing the top of ADR-0115's range. arm64 may stay behind x64 until BtbN's arm64 toolchain
can be trusted again. It is one patch behind today.

**Accepted gap, added to ADR-0115 §7's:** a breaking ffmpeg that shows up only on Windows is
seen when the pin is bumped, not when the release ships.
