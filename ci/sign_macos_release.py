#!/usr/bin/env python3
"""Sign and notarize a release's two macOS archives.

Two ways in, one procedure (ADR-0139):

  * `--archives DIR --out DIR`: `release.yml`'s `sign-macos` job, on a macOS
    runner, signs the archives the build legs just made, before anything is
    published. The identity comes from a `.p12` imported into a temporary
    keychain; notarytool authenticates with `--api-key/--api-key-id/--api-issuer`.
    Both live in the `release` environment's secrets.
  * a tag alone, or with `--upload`: the fallback on the dev's Mac (ADR-0138).
    It downloads an existing release's two macOS archives, signs them with the
    login keychain's identity and the `montagent-notary` profile (#585), and with
    `--upload` puts them back.

What it does, per `montagent-<tag>-{aarch64,x86_64}-apple-darwin.tar.gz`:

  * from a release, checks the downloaded archive against the release's own
    `SHA256SUMS`, so a corrupted download is never signed;
  * refuses a binary that already carries a Developer ID signature, so a second
    run cannot replace notarized bytes with a fresh, un-notarized signature;
  * signs with hardened runtime and a secure timestamp under the fixed identifier
    -- no entitlements, because #586 rendered (Skia plus the `ffmpeg` spawn)
    under `-o runtime` without any;
  * submits a zip of the binary to the notary service (it rejects `.tar.gz`,
    #584) and polls, because a first submission took about two hours (#586);
  * insists that Apple's log names the exact code hash that was signed;
  * repacks the same signed bytes under the same name and layout (#588), checks
    the new archive extracts to a tree identical to the signed one and that
    Apple holds a ticket for it, and (from a release) rewrites only the two
    macOS lines of `SHA256SUMS`.

Nothing touches the binary after `codesign`. A bare binary cannot be stapled,
so the ticket is fetched by Gatekeeper on first launch; #586 measured that as
silent for an MCP spawn.

With `--archives`, it writes the two signed archives to `--out` and stops; the
release job computes `SHA256SUMS` over them. From a release without `--upload`,
it stops with the new archives and `SHA256SUMS` in the work directory. With
`--upload`, it replaces those three assets on the release
(`gh release upload --clobber`, `SHA256SUMS` last) and downloads them again to
check the published bytes. `--clobber` deletes before it uploads, so for a
moment an asset is missing; sign before announcing the release.

Usage:
    sign_macos_release.py <tag> [--upload] [--profile montagent-notary]
    sign_macos_release.py <tag> --archives DIR --out DIR
                          --api-key KEY.p8 --api-key-id ID --api-issuer UUID
    common: [--workdir DIR] [--poll-seconds 60]

Exits 0 when both archives are signed and notarized (and written to `--out`,
or uploaded with `--upload`), 1 with a named defect otherwise.
"""

from __future__ import annotations

import argparse
import filecmp
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# Fixed forever once shipped: Gatekeeper and any user's tooling key on it, and a
# tool with no Info.plist otherwise takes its filename plus a hash (#584).
IDENTIFIER = "io.github.mbehtemam.montagent"
TEAM_ID = "8XTSMPCT3L"
TARGETS = {"aarch64-apple-darwin": "arm64", "x86_64-apple-darwin": "x86_64"}
DEFAULT_PROFILE = "montagent-notary"


class Defect(Exception):
    pass


def run(*cmd: str, check: bool = True, env: dict | None = None) -> subprocess.CompletedProcess:
    proc = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if check and proc.returncode != 0:
        raise Defect(f"`{' '.join(cmd)}` exited {proc.returncode}:\n{proc.stderr.strip() or proc.stdout.strip()}")
    return proc


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def log(msg: str) -> None:
    print(msg, flush=True)


def find_identity() -> str:
    """The SHA-1 of the one valid Developer ID Application identity for the team."""
    out = run("security", "find-identity", "-v", "-p", "codesigning").stdout
    found = re.findall(rf'([0-9A-F]{{40}}) "(Developer ID Application: [^"]*\({TEAM_ID}\))"', out)
    if len(found) != 1:
        raise Defect(
            f"expected exactly one valid 'Developer ID Application: … ({TEAM_ID})' identity in the "
            f"keychain, found {len(found)}. #585 records how it was created."
        )
    sha1, name = found[0]
    log(f"identity: {name} ({sha1})")
    return sha1


def notary_auth(args: argparse.Namespace) -> list[str]:
    """notarytool's credential flags: an API key file in CI, else the keychain profile."""
    if args.api_key:
        if not (args.api_key_id and args.api_issuer):
            raise Defect("--api-key needs --api-key-id and --api-issuer")
        auth = ["--key", str(args.api_key), "--key-id", args.api_key_id, "--issuer", args.api_issuer]
        what = f"API key {args.api_key_id}"
    else:
        auth = ["--keychain-profile", args.profile]
        what = f"keychain profile {args.profile}"
    proc = run("xcrun", "notarytool", "history", *auth, check=False)
    if proc.returncode != 0:
        raise Defect(
            f"{what} does not authenticate with the notary service:\n"
            f"{proc.stderr.strip()}\n#585 records how the credential was made."
        )
    log(f"notary credential: {what}")
    return auth


def read_sums(path: Path) -> list[tuple[str, str]]:
    """`sha256sum` lines as (hash, name), in file order."""
    rows = []
    for line in path.read_text().splitlines():
        m = re.fullmatch(r"([0-9a-f]{64}) [ *](.+)", line)
        if not m:
            raise Defect(f"SHA256SUMS line is not `<sha256>  <name>`: {line!r}")
        rows.append((m.group(1), m.group(2)))
    return rows


def signature(binary: Path) -> dict[str, str]:
    """`codesign -dvvv` as a dict, plus every Authority line joined."""
    proc = run("codesign", "-dvvv", str(binary), check=False)
    fields: dict[str, str] = {"returncode": str(proc.returncode)}
    authorities = []
    for line in proc.stderr.splitlines():
        key, sep, value = line.partition("=")
        if not sep:
            continue
        if key == "Authority":
            authorities.append(value)
        elif key == "CodeDirectory v":
            fields["flags"] = value
        else:
            fields[key] = value
    fields["Authority"] = "|".join(authorities)
    return fields


def tar_entries(archive: Path) -> list[str]:
    return sorted(e.rstrip("/") for e in run("tar", "-tzf", str(archive)).stdout.splitlines() if e.strip())


def trees_identical(a: Path, b: Path) -> bool:
    cmp = filecmp.dircmp(a, b)
    if cmp.left_only or cmp.right_only or cmp.funny_files:
        return False
    _, mismatch, errors = filecmp.cmpfiles(a, b, cmp.common_files, shallow=False)
    if mismatch or errors:
        return False
    for name in cmp.common_files:
        if os.stat(a / name).st_mode != os.stat(b / name).st_mode:
            return False
    return all(trees_identical(a / d, b / d) for d in cmp.common_dirs)


class Archive:
    def __init__(self, tag: str, target: str, work: Path):
        self.target = target
        self.arch = TARGETS[target]
        self.name = f"montagent-{tag}-{target}"
        self.file = f"{self.name}.tar.gz"
        self.src = work / "in" / self.file
        self.tree = work / "signed" / target
        self.binary = self.tree / self.name / "montagent"
        self.zip = work / "notary" / f"{self.name}.zip"
        self.out = work / "out" / self.file
        self.cdhash = ""
        self.submission = ""

    def extract(self) -> None:
        entries = tar_entries(self.src)
        if not entries or any(e != self.name and not e.startswith(self.name + "/") for e in entries):
            raise Defect(f"{self.file}: every entry must sit under `{self.name}/`, got {entries[:5]}…")
        self.entries = entries
        self.tree.mkdir(parents=True)
        run("tar", "-xzf", str(self.src), "-C", str(self.tree))
        if not self.binary.is_file():
            raise Defect(f"{self.file}: no `{self.name}/montagent` inside")
        archs = run("lipo", "-archs", str(self.binary)).stdout.split()
        if archs != [self.arch]:
            raise Defect(f"{self.file}: binary is {archs}, expected [{self.arch!r}]")
        if "Developer ID Application" in signature(self.binary)["Authority"]:
            raise Defect(
                f"{self.file}: the binary is already Developer ID signed. Re-signing would replace "
                f"notarized bytes with un-notarized ones; this release looks done already."
            )

    def sign(self, identity: str) -> None:
        run("codesign", "-s", identity, "-f", "--timestamp", "-o", "runtime", "-i", IDENTIFIER, str(self.binary))
        run("codesign", "--verify", "--strict", "-vv", str(self.binary))
        sig = signature(self.binary)
        problems = []
        if sig.get("Identifier") != IDENTIFIER:
            problems.append(f"Identifier={sig.get('Identifier')}")
        if sig.get("TeamIdentifier") != TEAM_ID:
            problems.append(f"TeamIdentifier={sig.get('TeamIdentifier')}")
        if "(runtime)" not in sig.get("flags", ""):
            problems.append(f"flags={sig.get('flags')} (no hardened runtime)")
        if "Timestamp" not in sig:
            problems.append("no secure timestamp")
        if problems:
            raise Defect(f"{self.file}: signature is wrong: {', '.join(problems)}")
        self.cdhash = sig["CDHash"]
        log(f"{self.target}: signed, CDHash {self.cdhash}")

    def submit(self, auth: list[str]) -> None:
        self.zip.parent.mkdir(parents=True, exist_ok=True)
        run("ditto", "-c", "-k", "--keepParent", str(self.binary), str(self.zip))
        out = run(
            "xcrun", "notarytool", "submit", str(self.zip),
            *auth, "--no-wait", "--output-format", "json",
        ).stdout
        self.submission = json.loads(out)["id"]
        log(f"{self.target}: submitted, notary id {self.submission}")

    def status(self, auth: list[str]) -> str:
        proc = run(
            "xcrun", "notarytool", "info", self.submission,
            *auth, "--output-format", "json", check=False,
        )
        if proc.returncode != 0:
            # A network blip is not a verdict; the next poll asks again.
            return "Unknown"
        return json.loads(proc.stdout)["status"]

    def check_log(self, auth: list[str]) -> None:
        notary_log = json.loads(
            run("xcrun", "notarytool", "log", self.submission, *auth).stdout
        )
        hashes = {entry.get("cdhash") for entry in notary_log.get("ticketContents") or []}
        if self.cdhash not in hashes:
            raise Defect(f"{self.target}: Apple's log does not name CDHash {self.cdhash} (it names {hashes})")
        if notary_log.get("issues"):
            raise Defect(f"{self.target}: accepted with issues:\n{json.dumps(notary_log['issues'], indent=2)}")

    def repack(self) -> None:
        self.out.parent.mkdir(parents=True, exist_ok=True)
        # bsdtar would otherwise add `._*` AppleDouble files for every xattr (the
        # binary carries `com.apple.provenance`), which the Linux-built original lacks.
        env = dict(os.environ, COPYFILE_DISABLE="1")
        run("tar", "--no-mac-metadata", "--no-xattrs", "-czf", str(self.out), "-C", str(self.tree), self.name, env=env)
        if tar_entries(self.out) != self.entries:
            raise Defect(f"{self.file}: the repacked archive's entries differ from the original's")
        check = self.out.parent.parent / "check" / self.target
        check.mkdir(parents=True)
        run("tar", "-xzf", str(self.out), "-C", str(check))
        if not trees_identical(self.tree, check):
            raise Defect(f"{self.file}: the repacked archive does not extract to the signed tree")
        run("codesign", "--verify", "--strict", str(check / self.name / "montagent"))
        if signature(check / self.name / "montagent").get("CDHash") != self.cdhash:
            raise Defect(f"{self.file}: the repacked binary's CDHash is not the notarized one")
        # Asks Apple online whether a ticket exists for this code hash. A copy signed
        # but never notarized fails it, so this proves what `lipo` cannot run here (an
        # x86_64 binary without Rosetta). `spctl --assess` and `syspolicy_check` are no
        # use: both reject every bare binary, notarized or not, for want of a bundle or
        # a stapled ticket.
        proc = run("codesign", "--verify", "--check-notarization", "-R=notarized",
                   str(check / self.name / "montagent"), check=False)
        if proc.returncode != 0:
            raise Defect(f"{self.file}: Apple reports no notarization ticket:\n{proc.stderr.strip()}")
        log(f"{self.target}: repacked, sha256 {sha256(self.out)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("tag", help="the release tag, e.g. v0.2.0")
    source = parser.add_mutually_exclusive_group()
    source.add_argument("--upload", action="store_true", help="replace the assets on the release")
    source.add_argument(
        "--archives", type=Path,
        help="sign the two archives in this directory instead of the release's (release.yml's "
             "sign-macos job); the signed archives are written to --out",
    )
    parser.add_argument("--out", type=Path, help="with --archives: where the signed archives go")
    parser.add_argument("--workdir", type=Path, help="defaults to a fresh temporary directory")
    parser.add_argument("--profile", default=DEFAULT_PROFILE, help="notarytool keychain profile")
    parser.add_argument("--api-key", type=Path, help="App Store Connect API key (.p8), instead of --profile")
    parser.add_argument("--api-key-id")
    parser.add_argument("--api-issuer")
    parser.add_argument("--poll-seconds", type=int, default=60)
    args = parser.parse_args()
    if args.archives and not args.out:
        raise Defect("--archives needs --out")

    tools = ["codesign", "ditto", "lipo", "security", "tar", "xcrun"] + ([] if args.archives else ["gh"])
    for tool in tools:
        if shutil.which(tool) is None:
            raise Defect(f"`{tool}` is not on PATH; this runs on macOS with Xcode installed")

    work = args.workdir or Path(tempfile.mkdtemp(prefix=f"montagent-sign-{args.tag}-"))
    if work.exists() and any(work.iterdir()):
        raise Defect(f"work directory {work} is not empty")
    log(f"work directory: {work}")

    identity = find_identity()
    auth = notary_auth(args)

    archives = [Archive(args.tag, target, work) for target in TARGETS]
    (work / "in").mkdir(parents=True, exist_ok=True)
    if args.archives:
        # Straight from the build legs: no SHA256SUMS exists yet, the release job
        # computes it over what this writes.
        sums = None
        for a in archives:
            if not (args.archives / a.file).is_file():
                raise Defect(f"{args.archives} has no `{a.file}`")
            shutil.copy2(args.archives / a.file, a.src)
    else:
        patterns = [a.file for a in archives] + ["SHA256SUMS"]
        run("gh", "release", "download", args.tag, "--dir", str(work / "in"),
            *[x for p in patterns for x in ("--pattern", p)])
        for p in patterns:
            if not (work / "in" / p).is_file():
                raise Defect(f"release {args.tag} has no asset `{p}`")
        sums = read_sums(work / "in" / "SHA256SUMS")
        listed = dict((name, digest) for digest, name in sums)
        for a in archives:
            if listed.get(a.file) != sha256(a.src):
                raise Defect(f"{a.file}: does not match its line in the release's SHA256SUMS")

    for a in archives:
        a.extract()
        a.sign(identity)
    for a in archives:
        a.submit(auth)

    pending = list(archives)
    started = time.monotonic()
    while pending:
        time.sleep(args.poll_seconds)
        for a in list(pending):
            status = a.status(auth)
            if status == "Accepted":
                a.check_log(auth)
                log(f"{a.target}: notarized after {(time.monotonic() - started) / 60:.0f} min")
                pending.remove(a)
            elif status in ("Invalid", "Rejected"):
                notary_log = run("xcrun", "notarytool", "log", a.submission, *auth, check=False)
                raise Defect(f"{a.target}: notarization {status}:\n{notary_log.stdout}")
        if pending:
            log(f"waiting on {', '.join(a.target for a in pending)} ({(time.monotonic() - started) / 60:.0f} min)")

    for a in archives:
        a.repack()

    if sums is None:
        args.out.mkdir(parents=True, exist_ok=True)
        for a in archives:
            shutil.copy2(a.out, args.out / a.file)
        log(f"signed archives written to {args.out}")
        return 0

    replaced = {a.file: sha256(a.out) for a in archives}
    new_sums = [(replaced.get(name, digest), name) for digest, name in sums]
    sums_out = work / "out" / "SHA256SUMS"
    sums_out.write_text("".join(f"{digest}  {name}\n" for digest, name in new_sums))
    log(f"SHA256SUMS: replaced {len(replaced)} of {len(sums)} lines")

    if not args.upload:
        log(f"\nNot uploaded. To publish:\n  gh release upload {args.tag} --clobber "
            + " ".join(str(a.out) for a in archives) + f" {sums_out}")
        return 0

    run("gh", "release", "upload", args.tag, "--clobber", *[str(a.out) for a in archives])
    run("gh", "release", "upload", args.tag, "--clobber", str(sums_out))
    back = work / "published"
    back.mkdir()
    run("gh", "release", "download", args.tag, "--dir", str(back),
        *[x for p in patterns for x in ("--pattern", p)])
    for p in patterns:
        if sha256(back / p) != sha256(work / "out" / p):
            raise Defect(f"published `{p}` is not the bytes this script produced")
    log(f"uploaded to {args.tag} and read back: identical")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Defect as defect:
        print(f"error: {defect}", file=sys.stderr)
        sys.exit(1)
