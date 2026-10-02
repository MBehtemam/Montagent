# What Apple requires to notarize a bare command-line binary, and what a quarantined copy does afterwards

Research for [#584](https://github.com/MBehtemam/Montagent/issues/584), part of map
[#583](https://github.com/MBehtemam/Montagent/issues/583) (the macOS release binaries ship signed and
notarized). Builds on [#360](https://github.com/MBehtemam/Montagent/issues/360), which measured what
macOS 27.0 does to the **unsigned** binary.

Montagent ships as a bare Mach-O CLI binary (no `.app`) in a `.tar.gz`, usually spawned by an MCP
client. Sources are restricted to Apple developer documentation, man pages shipped on this machine
(macOS 27.0, Xcode command-line tools), and Apple DevForums posts by Apple DTS staff (Quinn "The
Eskimo!"). Nothing was signed or notarized for this note. Anything the sources leave open is marked
**needs the hand-signed test** ([#586](https://github.com/MBehtemam/Montagent/issues/586)).

## Summary

| # | Question | Answer |
|---|---|---|
| 1 | Certificate and `codesign` flags | **Developer ID Application**. `codesign -s "Developer ID Application: …" -f --timestamp -o runtime -i <reverse-DNS id> montagent`. No `--deep`. |
| 2 | Hardened-runtime entitlements | **Probably none.** The binary links only Apple system libraries, so library validation passes; spawning `ffmpeg` is a separate executable and inherits nothing. JIT / unsigned executable memory: no doc settles it for Skia's CPU path. **Needs the hand-signed test**: sign with `-o runtime` and no entitlements, then run a full render. |
| 3 | Upload container and auth | `notarytool submit` takes **zip, signed flat `.pkg`, or UDIF `.dmg` only**, not `.tar.gz`. Zip the signed binary just for submission, then ship the *same bytes* in the `.tar.gz`. Auth: App Store Connect API key, Apple ID + app-specific password, or keychain profile. **Recommended: a Team App Store Connect API key** (`-k/-d/-i`); it is the only one with no Apple ID password and no login-keychain dependency, so it works headless. |
| 4 | Stapling and first exec | **Stapling a bare binary is impossible** (Apple says so outright). Online, the first exec fetches the ticket from Apple's servers by cdhash and lets the code load. **Offline, the code is blocked.** Whether a notarized CLI shows any first-launch dialog when exec'd, and whether an MCP-spawned child stalls or dies like #360: **needs the hand-signed test**. |
| 5 | Does a `.pkg` change it? | Yes. A Developer ID Installer–signed, notarized, **stapled** `.pkg` works offline, and files the Installer lays down are **not quarantined**, so the tool's own exec gate never runs. It installs wherever the package says (e.g. `/usr/local/bin`); it needs the **Developer ID Installer** certificate as well. |

## 1. Certificate and `codesign` flags

**Certificate: Developer ID Application.** Apple's notarization requirements say to "use a 'Developer
ID' application, kernel extension, system extension, or installer certificate", and not a Mac
Distribution, ad hoc, Apple Development or local certificate
([Notarizing macOS software before distribution](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)).
Which one is named specifically: "When code signing items like Mach-O files, disk images, bundles,
apps, command line tools … sign with a Developer ID Application certificate. Sign installer packages
with a Developer ID Installer certificate"
([Resolving common notarization issues](https://developer.apple.com/documentation/security/resolving-common-notarization-issues#Use-a-valid-Developer-ID-certificate)).

Only the team's **Account Holder** can create Developer ID certificates, up to five of each type
([Create Developer ID certificates](https://developer.apple.com/help/account/certificates/create-developer-id-certificates/)).

**Flags.** Notarization requires, for every executable
([Notarizing macOS software before distribution](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution#Prepare-your-software-for-notarization)):

- a valid Developer ID signature;
- the **Hardened Runtime** (`-o runtime`). Without it the notary log says "The executable does not
  have the hardened runtime enabled"
  ([Resolving common notarization issues](https://developer.apple.com/documentation/security/resolving-common-notarization-issues#Enable-the-hardened-runtime));
- a **secure timestamp** (`--timestamp`). Without it: "The signature does not include a secure
  timestamp". Timestamping needs internet access to `timestamp.apple.com`
  ([same page](https://developer.apple.com/documentation/security/resolving-common-notarization-issues#Include-a-secure-timestamp)).
  `man codesign`: with `--timestamp` and no value "a default server provided by Apple is used … If
  the timestamp authority service cannot be contacted … the signing operation will fail";
- no `com.apple.security.get-task-allow` entitlement;
- linked against the macOS 10.9 SDK or later.

Quinn's canonical command for a non-bundled executable (a daemon, which has the same shape as a CLI
tool) in [Creating Distribution-Signed Code for Mac](https://developer.apple.com/forums/thread/701514):

```sh
codesign -s "Developer ID Application" -f --timestamp -o runtime -i "com.example.apple-samplecode.DaemonWithApp.Daemon" "to-be-signed/Daemon"
```

- `-i`: Quinn treats it as required for a bundle-less executable: the identifier is "the bundle ID
  the code would have if it had a bundle ID". `man codesign` explains why: without `-i` and without
  an Info.plist the identifier is "derived from … the filename of the executable", and `--prefix`
  exists because "command tools without Info.plists … default identifier is simply the command's
  filename". Today's linker signature carries `Identifier=montagent-89d90495d8bb3f34` (observed
  with `codesign -dv target/release/montagent`). Pick a stable reverse-DNS id and use it on both
  architectures.
- `-f`: needed because the Rust linker already ad-hoc signs the binary (`flags=0x20002(adhoc,linker-signed)`,
  observed). `man codesign` says linker signatures "can be replaced without using the --force option",
  so `-f` is harmless rather than strictly needed.
- **No `--deep`**: Quinn's post has a section headed "--deep Considered Harmful". It is irrelevant
  for a single Mach-O anyway.
- `--entitlements`: only if question 2 turns out to need one.

Self-check before uploading: `codesign -vvv --deep --strict <path>`, and `codesign -dvv <path>` should
show a `Timestamp=` line, not `Signed Time`
([Resolving common notarization issues](https://developer.apple.com/documentation/security/resolving-common-notarization-issues#Include-a-secure-timestamp)).
After notarization, `codesign -vvvv -R="notarized" --check-notarization <path>` covers "other code"
that is not an app, dmg or pkg ([Testing a Notarised Product](https://developer.apple.com/forums/thread/130560)).

## 2. Hardened-runtime entitlements for Skia and spawning `ffmpeg`

The Hardened Runtime "disallow[s] certain less common capabilities, like just-in-time (JIT)
compilation"; you relax one only with an entitlement, and should "use only the entitlements that are
absolutely necessary" ([Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime)).
`man codesign` (`runtime` flag): "runtime code signing enforcement, library validation, hard, kill,
and debugging restrictions". The triggers, one by one:

| Exception entitlement | What triggers it (Apple's words) | Does Montagent hit it? |
|---|---|---|
| [`com.apple.security.cs.allow-jit`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-jit) | Creating writable+executable memory with `mmap(MAP_JIT)`; examples are JavaScriptCore's fast path, "certain Python frameworks", PCRE JIT. | No JIT engine in the dependency graph (no JS engine, no wasm runtime). Skia runs CPU-only here (`skia-safe` features `binary-cache, embed-icudtl, pdf, jpeg`; no `gl`, per ADR-0010). The binary imports `_mmap` and `_mprotect` (observed with `nm -u`), which Rust's std uses too. Static inspection cannot tell whether any call asks for executable pages. **Needs the hand-signed test.** |
| [`com.apple.security.cs.allow-unsigned-executable-memory`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-unsigned-executable-memory) | Writable+executable memory *without* `MAP_JIT`: patching C code, `NSCreateObjectFileImageFromMemory`, DVDPlayback. | Not expected. Same test settles it. |
| [`com.apple.security.cs.allow-dyld-environment-variables`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-dyld-environment-variables) | The program relies on `DYLD_*` variables at runtime. | No. Montagent does not depend on `DYLD_*`. Under the hardened runtime dyld simply ignores them for this process. |
| [`com.apple.security.cs.disable-library-validation`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.disable-library-validation) | Loading "frameworks, plug-ins, or libraries unless they're either signed by Apple or signed with the same Team ID". Apple warns that Gatekeeper "runs extra security checks" on programs that disable it. | **No.** `otool -L target/release/montagent` lists only `/usr/lib/libc++`, `libiconv`, `libSystem` and the `ApplicationServices`, `CoreFoundation`, `CoreGraphics` and `CoreText` system frameworks: all Apple-signed. Skia is statically linked into the binary, so it is covered by Montagent's own signature. |

**Spawning `ffmpeg`.** Entitlements attach to executables, and only "Shared libraries, frameworks, and
in-process plug-ins inherit the entitlements of their host executable"
([Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime)). An `ffmpeg`
started with `posix_spawn`/`Command` is a separate executable with its own signature (or none). It
is not in-process, so it neither needs nor inherits anything from Montagent. None of the Hardened
Runtime's restrictions as Apple lists them covers launching a child process. Whether the user's
`ffmpeg` is itself quarantined is that binary's business, not Montagent's.

So the expected entitlement set is **empty**, which means no `--entitlements` file at all. The
[Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime) page says not
to include a false-valued entitlement. Confirm it in #586 by signing with `-o runtime` and no
entitlements, then running a real render end to end (Skia raster, PDF/JPEG paths, and the `ffmpeg`
spawn). A hardened-runtime violation shows up as a crash or a code-signing kill, not a quiet
fallback.

## 3. What `notarytool submit` accepts, and how it authenticates

**Containers.** `man notarytool`, SUPPORTED UPLOAD FILE FORMATS: "notarytool submit works only with
UDIF disk images, signed "flat" installer packages, and zip files … Passing any other file format …
will result in an error." Same list in
[Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow#Export-a-package-for-notarization).
**A `.tar.gz` is not accepted.**

That does not force the shipped format to change:

- A ticket is "a list of cdhash values that's been signed by Apple", and the trusted execution system
  looks the ticket up by the code's cdhash
  ([Notarisation Fundamentals](https://developer.apple.com/forums/thread/710738), Quinn). The ticket
  belongs to the signed Mach-O's code, not to the container it was uploaded in.
- Apple's own flow notarizes a zip, then re-packs for distribution: "While you can notarize a ZIP
  archive, you can't staple to it directly. Instead, run stapler against each item … Then create a new
  ZIP file containing the stapled items for distribution"
  ([Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow#Staple-the-ticket-to-your-distribution)).
  Quinn says the same: put native code "into a supported container like a zip archive to notarise it,
  then rearrange the code into some final distribution format"
  ([Notarisation Fundamentals](https://developer.apple.com/forums/thread/710738)).

So the pipeline is: sign → `ditto -c -k --keepParent montagent montagent.zip` (the `ditto` form Apple
shows) → `notarytool submit montagent.zip --wait` → put the **same signed bytes** into the `.tar.gz`
→ compute `SHA256SUMS` over the final archives. Anything that rewrites the binary after signing breaks
the cdhash match (strip, re-link, re-sign). One submission can carry both architectures' binaries in
one zip, or each can go separately; Apple asks you to stay under 75 notarizations a day
([same page](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow#Avoid-long-notarization-response-times-and-size-limits)).
Always pull `notarytool log <id>` even on `Accepted`, since warnings land there.

**Authentication** (`man notarytool`, AUTHENTICATION OPTIONS, and `xcrun notarytool submit --help`):

| Method | Flags | Headless? | Notes |
|---|---|---|---|
| App Store Connect API key | `-k <path to .p8> -d <key id> [-i <issuer id>]` | **Yes**: no prompt, no keychain, no Apple ID session. | Team keys need `-i`. An individual key must *not* pass `-i` ("will result in a "401 Unauthorized""). The private key "can only be downloaded once" ([Creating API keys](https://developer.apple.com/documentation/appstoreconnectapi/creating-api-keys-for-app-store-connect-api)). The same key also drives the REST [Notary API](https://developer.apple.com/documentation/notaryapi/submitting-software-for-notarization-over-the-web), so it works without Xcode. |
| Apple ID + app-specific password | `--apple-id --team-id [--password]` | Only if the password is passed on the command line or from env, which is a cleartext secret. With `--password` omitted it gives "a secure prompt", which a headless run can't answer. | App-specific password per [Using app-specific passwords](https://support.apple.com/en-us/HT204397) (the link `man notarytool` gives). Apple's docs suggest the keychain profile precisely "to avoid including your password as cleartext in a script" ([Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow#Upload-your-app-to-the-notarization-service)). |
| Keychain profile | `store-credentials <name>` once, then `-p <name> [--keychain <path>]` | Only on a Mac whose keychain is unlocked. `--keychain`: "If the specified keychain file is locked, you will be prompted to unlock it." | Wraps either of the above. `store-credentials` saves "to the default login keychain", and with `-k` it copies the key's contents in so the `.p8` "can be deleted". Good for a human's laptop. On CI it means creating and unlocking a temporary keychain first. |

**Team key vs individual key.** Apple's sources disagree here. `man notarytool` says notarytool
"supports two types of App Store Connect API Keys. Team Keys and Individual Keys". But Apple's
[Creating API keys for App Store Connect API](https://developer.apple.com/documentation/appstoreconnectapi/creating-api-keys-for-app-store-connect-api)
says "Individual keys aren't able to use Provisioning endpoints, access sales-and-finance, or
`notaryTool`." A **Team key** works by both accounts. Generating one needs an App Store Connect
**Admin** ("To generate team keys, you must have an Admin account in App Store Connect"). Which minimum *role* the team key needs for notarization is not
stated on the pages read here. **Needs the hand-signed test** (#585 picks the role; #586 proves it).

**Recommendation for #585: a Team App Store Connect API key** used directly with `-k/-d/-i`. Reasons:

1. It is the only method that works headless with no interactive step, no Apple ID password and no
   unlocked login keychain. It suits any answer to #587 (a local script, a self-hosted runner, or GitHub
   Actions once billing is back).
2. It is revocable on its own, without touching the Apple ID
   ([Creating API keys](https://developer.apple.com/documentation/appstoreconnectapi/creating-api-keys-for-app-store-connect-api)).
3. It avoids the man-page vs. App Store Connect conflict over individual keys.

Store the `.p8` outside the repo. On the owner's Mac it can go into a keychain profile
(`store-credentials … -k … -d … -i …`). In CI it goes in an encrypted secret. Record only *where*
it lives. The ticket's never-commit rule stands.

## 4. Stapling, and what a quarantined, notarized, unstapled binary does

**Stapling a bare binary is impossible.** This is confirmed in Apple's own words: "Although tickets
are created for standalone binaries, it's not currently possible to staple tickets to them"
([Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow#Staple-the-ticket-to-your-distribution)).
`man stapler`: "stapler works only with UDIF disk images, signed "flat" installer packages, and
certain code-signed executable bundles such as ".app"." A zip or `.tar.gz` can't be stapled either.

**Online first exec.** The ticket is published online "where Gatekeeper can find it", and "the next
time any user attempts to run your app … Gatekeeper finds the ticket online. This includes users who
downloaded your app before notarization"
([Notarizing macOS software](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution),
[Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow#Staple-the-ticket-to-your-distribution)).
Quinn describes the mechanism. The trusted execution system "fails to find the cdhash in its local
database, reaches out to Apple's servers, downloads the ticket, ingests it into the local cache, and
allows the code to load". The cost is "a slight delay" on first launch
([The Pros and Cons of Stapling](https://developer.apple.com/forums/thread/720093)). The lookup is a
CloudKit record fetch keyed by cdhash (`com.apple.gk.ticket-delivery`;
[Notarisation Fundamentals](https://developer.apple.com/forums/thread/710738)). `stapler` itself
needs the CloudKit IP ranges on port 443
([Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow#Ensure-your-build-server-has-network-access)).
Quinn notes it is the trusted execution system, so the check is not tied to app bundles. In "macOS 12
and earlier, this is done by Gatekeeper only when the code is quarantined … This is changing in
macOS 13" ([Notarisation Fundamentals](https://developer.apple.com/forums/thread/710738)).

**Offline first exec: blocked.** If the Mac is offline, the system "fails to find the cdhash in the
local database, fails to fetch the ticket … [and] blocks the code from loading"
([The Pros and Cons of Stapling](https://developer.apple.com/forums/thread/720093)). Quinn's
[Packaging Mac Software for Distribution](https://developer.apple.com/forums/thread/701581) says the
same: without a staple "a user might find that your product is blocked by Gatekeeper if they try to
install or use it while the Mac is offline". After one successful online exec, the ticket sits in the
local cache, so later offline runs should be fine
([The Pros and Cons of Stapling](https://developer.apple.com/forums/thread/720093)). *How* the offline
block shows up for an exec'd CLI is not documented: #360's dialog-then-SIGKILL, a silent SIGKILL, or
something else. Nor is whether that denial is cached the way #360's was. **Needs the hand-signed
test** (network off, fresh quarantined copy).

**Any dialog for a notarized CLI?** For apps, Apple documents a first-open prompt: "The first time that
you open a new app from an identified developer … your Mac asks if you're sure that you want to open
it" ([Safely open apps on your Mac](https://support.apple.com/en-us/102445)). Gatekeeper "requests
user approval before opening downloaded software for the first time"
([Gatekeeper and runtime protection](https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-sec5599b66df/web)).
For **command-line tools**, the only Apple-staff statement found is Quinn's: running a correctly
signed and notarised tool from Terminal "runs through the standard Gatekeeper check, which passes
because the tool is correctly signed and notarised"
([thread 689337](https://developer.apple.com/forums/thread/689337)). He does not say whether "passes"
means silently, or after a one-time approval prompt. The two documented failure modes for tools are
both about **Finder double-click**, which runs Gatekeeper's *document* logic and "always fails no
matter how well signed or notarised the tool is" (r. 58097824;
[Resolving Gatekeeper Problems](https://developer.apple.com/forums/thread/706379),
[thread 689337](https://developer.apple.com/forums/thread/689337)). An MCP client exec's the binary
and never double-clicks it, so that bug should not apply.

So, would the MCP-spawned child be blocked the way #360's unsigned one was? The docs point to **no
while online**: the exec gate finds a valid Developer ID signature plus an online ticket. But no Apple
source states that a quarantined, notarized, *non-bundle* executable gets **no** first-launch approval
prompt on macOS 15+. If it does, an MCP-spawned child would hang in `execve` as in #360 until someone
answers. **Needs the hand-signed test**, which is exactly #586: browser download → `tar xzf` → spawn
from a parent process, online and offline, first and second run.

**A source conflict worth knowing about.** Quinn has written that "Unix-y unarchiving tools, like tar
and unzip, don't propagate quarantine" ([forums](https://developer.apple.com/forums/thread/706379)).
#360 observed the opposite on macOS 27.0: `bsdtar` *did* propagate quarantine onto the extracted
binary, verified with a control. Trust the measurement, and have #586 assume the extracted binary is
quarantined.

## 5. Does a `.pkg` change any of this?

Yes, on three points:

1. **It can be stapled.** `man stapler` lists signed flat installer packages as stapleable. A stapled
   `.pkg` lets Gatekeeper verify "offline". `productbuild` timestamps Developer ID signatures by
   default (`man productbuild`, SIGNED PRODUCT ARCHIVES). The installed tool still needs its *own*
   Developer ID Application signature and notarization. For a custom installer Apple asks you to
   "notarize the installer's payload … then package the notarized items into the installer and
   notarize it", but a submitted package's nested code is ticketed in the same submission: "The
   notary service generates a ticket for the top-level file … as well as for each nested file"
   ([Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow#Upload-your-app-to-the-notarization-service)).
2. **The installed binary is not quarantined.** "If you use an installer package to install the tool,
   it's no longer quarantined and thus avoids this check"
   ([Quinn, thread 689337](https://developer.apple.com/forums/thread/689337)). And: "When the user goes
   to install the package, Gatekeeper checks it. Assuming that check passes, Gatekeeper does no further
   checks on the content it installed"
   ([Resolving Gatekeeper Problems](https://developer.apple.com/forums/thread/706379)). This removes
   question 4's open risk (a prompt on the spawned child), because the only check happens once, at
   install time, in front of a human.
3. **It needs a second certificate**: **Developer ID Installer** for the `.pkg`
   ([Resolving common notarization issues](https://developer.apple.com/documentation/security/resolving-common-notarization-issues#Use-a-valid-Developer-ID-certificate);
   [Packaging Mac Software for Distribution](https://developer.apple.com/forums/thread/701581)). It is
   also Account-Holder-created
   ([Create Developer ID certificates](https://developer.apple.com/help/account/certificates/create-developer-id-certificates/)).

**Where it installs.** Wherever the package says. `pkgbuild --root <dir> --install-location <path>`
sets the default ("you might specify an install-path of /Applications"), and
`productbuild --package <pkg> [install-path]` can override it (`man pkgbuild`, `man productbuild`).
For a CLI the conventional choice is `/usr/local/bin`. Apple does not mandate it, and Quinn's
`productbuild` example targets `/Applications` because it installs an app
([Packaging Mac Software for Distribution](https://developer.apple.com/forums/thread/701581)). Writing
there means the macOS Installer asks for an administrator password. That is a UX cost the `.tar.gz`
path does not have, and a decision for #588.

**What a `.pkg` does not change**: the binary's own signing (Q1) and entitlements (Q2) are identical,
and the `.tar.gz` downloaded in a browser keeps every caveat in Q4.

## Consequences for the sibling tickets

- **#585 (certificate and credential):** Required: one **Developer ID Application** certificate
  (Account Holder creates it). **Developer ID Installer** is needed **only if #588 picks `.pkg`**.
  Notary credential: a **Team App Store Connect API key** (`.p8` + key ID + issuer ID), created by an
  Admin. Record where the `.p8` and the signing identity live, never their values.
- **#586 (hand-signed test):** Must settle: (a) no entitlements needed under `-o runtime` for a real
  render plus `ffmpeg` spawn; (b) whether a quarantined, notarized, unstapled binary spawned by a
  parent shows any prompt or delay on the first run, online; (c) what happens on the first run
  **offline** (dialog? silent SIGKILL? cached denial that survives going back online?); (d) the
  minimum App Store Connect role a Team key needs for `notarytool`; (e) that a `.tar.gz` re-pack of
  the zip-notarized binary still passes `--check-notarization`.
- **#587 (where signing runs):** It needs macOS (`codesign`, `notarytool` via `xcrun`), internet to
  `timestamp.apple.com` and the notary/S3 endpoints, and the signing identity in an unlocked keychain.
  The API key keeps the *notary* half headless anywhere. The *codesign* half still needs the Developer
  ID identity in a keychain on that Mac. Signing must happen **before** `SHA256SUMS` is computed, and
  nothing may touch the binary between signing and archiving.
- **#588 (binary vs `.pkg`):** The `.pkg` is the only format Apple documents as fully closing both
  gaps: it can be stapled (offline first install works) and the installed tool is never quarantined
  (no exec-time gate on the MCP-spawned child). Its costs are the second certificate, an admin password
  at install, and a different install story from the README's "extract and point your client at it".
  The bare `.tar.gz` stays viable if #586 shows that online first exec is silent. The offline first
  run is then a known, documentable gap.

## Needs the hand-signed test (#586)

1. Whether the Skia CPU raster path or anything else in the binary needs `allow-jit` /
   `allow-unsigned-executable-memory` under the hardened runtime. Expected: no.
2. Whether a quarantined, notarized, unstapled CLI exec'd by a parent gets any first-launch prompt on
   macOS 15+/27, online.
3. The exact offline first-exec behaviour, and whether it is cached.
4. The minimum App Store Connect role a Team API key needs for notarization.
5. That re-packing the zip-notarized binary into `.tar.gz` keeps it notarized. Expected yes, since the
   ticket is per-cdhash.
