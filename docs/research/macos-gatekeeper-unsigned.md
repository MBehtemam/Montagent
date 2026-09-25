# What macOS does to an unsigned, un-notarized Montagent — and the one correct README instruction

Research for [#360](https://github.com/MBehtemam/Montagent/issues/360), part of the map [#356](https://github.com/MBehtemam/Montagent/issues/356).

**Date of research:** 2026-09-25.

**Test machine:** macOS **27.0**, build **26A428**, `Darwin 27.0.0` … `RELEASE_ARM64_T6000` (Apple silicon).
Captured with `sw_vers` and `uname -a`. Every "observed" claim below was reproduced on this
machine on this date; it is a single machine on one OS version, and that is the limit of the
empirical evidence.

**Sourcing rule applied:** claims are either (a) **observed** — a command run on the test machine,
with its real output pasted in — or (b) **documented** — cited to Apple's own page, with a URL, or
to a man page shipped on this machine. Anything I could not establish either way is collected under
[Not verified](#not-verified) rather than guessed at.

---

## The short answer

1. The quarantine attribute **does** survive `tar xzf`. macOS's `tar` is `bsdtar`/libarchive and
   propagates `com.apple.quarantine` from the archive onto every extracted file. The extracted
   binary is quarantined.
2. A quarantined, non-notarized binary is **blocked at `execve`**, not at Finder double-click.
   Running it from a plain shell raises a full-screen-modal system dialog and the exec **hangs**
   until a human answers it.
3. If the human answers with the only non-destructive button (**Done**), the exec is **denied** and
   the process is **`SIGKILL`ed — exit 137**, with **empty stdout and empty stderr**.
4. **Every later exec of that same binary is `SIGKILL`ed immediately, with no dialog at all.** This
   is the case that matters: an MCP client spawning `montagent` gets a pid that dies instantly and
   silently.
5. The minimal correct escape is **`xattr -d com.apple.quarantine ./montagent`**. It works, it works
   *before* the first launch (no need to be blocked first), it needs no GUI, no `sudo`, and no trip
   to System Settings. `spctl --add` has been **removed**; Control-click → Open has been **removed**;
   "Open Anyway" in System Settings works but is GUI-only and cannot help a subprocess.

---

## 1. What the user actually sees

### The dialog, verbatim

Observed on this machine (macOS 27.0) when a quarantined, non-notarized binary was executed. This is
the text as read off the screen:

> **"montagent" Not Opened**
>
> Apple could not verify "montagent" is free of malware that may harm your Mac or compromise your
> privacy.
>
> \[ **Move to Trash** ]
> \[ **Done** ]

A `?` help button sits at the top right.

Two things are **not** in this dialog, and both matter for the README:

- There is **no "Open Anyway" button**. Any instruction of the form "click Open Anyway in the dialog"
  is wrong on this OS.
- There is **no "Cancel"**. The two buttons are *destroy the file* and *give up*.

This is the modern blocking dialog, not the older two-button **Cancel / Open** alert that most
third-party write-ups still describe. Apple's own support page shows the same shape — an alert
reading *"Apple cannot check 'Example App' for malicious software"* offering **Move to Trash** or
**Done**
([Safely open apps on your Mac](https://support.apple.com/en-us/102445)).

### Which launch produced it

**A shell exec — not Finder.** The dialog above was raised by `./montagent` run from `zsh`. The
unified log names the subsystem explicitly:

```
$ log show --last 25m --predicate 'eventMessage CONTAINS[c] "montagent"' --style compact
…
syspolicyd[512] [com.apple.syspolicy.exec:default] GK evaluateScanResult: 1,
    PST: (path: 2711a2ebfea1b47), (team: (null)), (id: montagent), (bundle_id: NOT_A_BUNDLE), 1, 0, 1, 0, 0, 0, 0
syspolicyd[512] [com.apple.syspolicy.exec:default] Prompt shown (6, 0), waiting for response:
    PST: (path: 2711a2ebfea1b47), (team: (null)), (id: montagent), (bundle_id: NOT_A_BUNDLE)
```

`com.apple.syspolicy.exec` is the **exec** gate. `bundle_id: NOT_A_BUNDLE` confirms this path is for
a bare Mach-O, not an app bundle. And `Prompt shown … waiting for response` is the literal statement
that **the exec blocks on a human**.

The same log shows the control: the *non*-quarantined copy of the identical binary got
`GK evaluateScanResult: 2` and no prompt.

Note also what the block is *not*. AMFI logs

```
kernel (AppleMobileFileIntegrity) AMFI: '…/montagent' has no CMS blob?
kernel (AppleMobileFileIntegrity) AMFI: '…/montagent': Unrecoverable CT signature issue, bailing out.
```

for **both** copies — quarantined and not — and the non-quarantined one ran fine. So AMFI's
complaint about the missing CMS blob is noise here, not the gate. **Quarantine is the gate.**

### A note on "unsigned"

Montagent's release binary will not literally be unsigned. On Apple silicon the linker ad-hoc signs
every Mach-O it produces, and the test binary used here (plain `cc`, the same as `rustc`'s default)
reports:

```
$ codesign -dvvv montagent
CodeDirectory v=20400 size=386 flags=0x20002(adhoc,linker-signed) …
Signature=adhoc
TeamIdentifier=not set
```

So "unsigned and un-notarized" precisely means **ad-hoc signed, no Developer ID, no notarization
ticket** — which is what Gatekeeper rejects.

---

## 2. Does quarantine survive `tar xzf`?

**Yes. It propagates from the archive to every extracted file.** This is the single most commonly
mis-stated fact in third-party guides, so here is the whole experiment.

`tar` on this machine is libarchive, which is quarantine-aware:

```
$ /usr/bin/tar --version
bsdtar 3.5.3 - libarchive 3.7.4 zlib/1.2.12 liblzma/5.4.3 bz2lib/1.0.8
$ ls -l /usr/bin/tar
lrwxr-xr-x  1 root  wheel  6 Sep  3 12:34 /usr/bin/tar -> bsdtar
```

**Experiment — quarantined archive.** A browser download was simulated by writing the attribute the
way Safari writes it (`xattr -w`, per [`man xattr`](#man-pages) form three):

```
$ xattr -w com.apple.quarantine "0081;6ab61995;Safari;413FE092-…" montagent-v0.1.0-aarch64-apple-darwin.tar.gz
$ mkdir extract && cd extract && tar xzf ../montagent-v0.1.0-aarch64-apple-darwin.tar.gz
$ xattr -l montagent
com.apple.provenance:
com.apple.quarantine: 0081;6ab61995;;413FE092-8C91-42B3-956A-3201037BE8AF
$ ls -l@ montagent
-rwxr-xr-x@ 1 mohammedehtemam  wheel  33432 Sep 25 08:49 montagent
	com.apple.provenance	   11
	com.apple.quarantine	   51
```

The attribute lands on the **extracted binary**, not only the archive. The flags (`0081`) and the
UUID carry over verbatim; the *agent name* field is blanked (`Safari` → empty) and, in a repeat run,
the timestamp field was re-stamped to extraction time. Neither affects the outcome.

**Control — non-quarantined archive.** Same archive, attribute stripped first:

```
$ cp …tar.gz clean.tar.gz && xattr -c clean.tar.gz
$ mkdir e4 && cd e4 && tar xzf ../clean.tar.gz
$ xattr -l montagent
com.apple.provenance:
$ ./montagent
montagent ok          # exit 0
```

No quarantine in, no quarantine out, and it runs. So the extracted binary's quarantine comes
**strictly** from the archive's.

**Corollary — `curl` does not quarantine.** Verified on this machine: a file written by
`/usr/bin/curl` carries only `com.apple.provenance`, never `com.apple.quarantine`. Apple staff state
the same on the developer forums: *"Most Unix-y tools don't quarantine their downloads, including
curl and scp"* ([Apple Developer Forums, thread 683551](https://developer.apple.com/forums/thread/683551)).
**A `curl`-based install line sidesteps this entire problem.** That is a real option for the README,
recorded here for [#356](https://github.com/MBehtemam/Montagent/issues/356) to weigh — but it does
not help the user who clicked the release link in a browser, which is what this ticket asks about.

---

## 3. What the parent process observes — the MCP subprocess case

This is the load-bearing finding. Full probe, run unattended from a shell script:

```
== A: quarantined binary, FIRST exec ==
   com.apple.quarantine: 0081;6ab61995;;413FE092-…
   stdout/err: []
   exit=137  elapsed=108s     -> signal 9

== B: SAME binary, SECOND exec (post-block) ==
   com.apple.quarantine: 0081;6ab61995;;413FE092-…
   stdout/err: []
   exit=137  elapsed=2s       -> signal 9

== C: xattr -d then exec ==
   xattr -d rc=0
   stdout/err: [montagent ok]
   exit=0    elapsed=0s

== D: spctl assess after clearing quarantine ==
   montagent: rejected
   spctl rc=3
```

Read that carefully:

**A — the first exec hangs, then dies.** 108 seconds elapsed: that is the dialog sitting on screen
waiting. There is **no timeout of its own that I could establish** — the 108s is how long the human
took, not a system limit (see [Not verified](#not-verified)). While blocked, the child **does exist**
in the process table:

```
$ pgrep -P <shell-pid> | …
  PID  PPID STAT ELAPSED COMMAND
21830 21821 SN     01:37 ./montagent
```

State `S` — interruptible sleep, blocked inside `execve`, no process image loaded, no code of ours
ever ran. **`posix_spawn`/`fork` succeeded and returned a valid pid.** A parent that checks "did the
spawn succeed?" sees *yes*.

When the human clicks **Done**, the exec is denied and the child is **`SIGKILL`ed: exit 137, signal
9, stdout empty, stderr empty.** The parent gets no error message, no exit text, nothing to log —
only a status word.

**B — and thereafter it is silent.** The *second* exec of the same binary took **2 seconds** and was
`SIGKILL`ed with **no dialog at all**. The denial is remembered. So the intuitive recovery — "run it
again and click through the prompt this time" — **does not work**; the second run just dies.

**What an MCP client actually experiences.** Montagent is spawned as a stdio subprocess. The client
writes `initialize` and waits for a response on stdout. In case A it waits on a pipe that will never
produce a byte, for as long as a dialog the client's user may never see stays up — then gets EOF and
a 137. In case B it gets EOF and a 137 within ~2 seconds. Either way the observable symptom is
**"the MCP server failed to start"** or a handshake timeout, with **no diagnostic text whatsoever**,
because the binary was killed before `main` ran and could not write a word.

This is precisely the silent failure #360 flagged. It is worse than the human case, because the
human at least gets a dialog explaining itself.

### Terminal vs. a GUI-launched MCP client

The gate is the same one — `com.apple.syspolicy.exec` — in both cases, because it is keyed on the
*file's* quarantine attribute and the exec syscall, not on who the parent is. The difference is only
in whether anyone sees the prompt:

- **From Terminal:** the dialog appears (observed). The user can connect it to the command they just
  typed. Answering **Done** → `SIGKILL`.
- **From a GUI desktop MCP client:** the same exec gate fires. The dialog is raised by `syspolicyd`
  into the user's GUI session, so it *can* appear — but it names `montagent`, a binary the user never
  launched by hand and may not recognise, while the client itself only reports a failed server. The
  connection between the two is not made for the user.
- **Second and subsequent launches (the steady state):** **no dialog in either case.** Immediate
  `SIGKILL`. This is what a user who already dismissed one prompt will actually live with.

---

## 4. The escapes — which still work

| Escape | Status on macOS 27.0 | Evidence |
|---|---|---|
| `xattr -d com.apple.quarantine <path>` | **Works.** Non-GUI, no `sudo`, works *before* first launch. | Observed, below |
| System Settings → Privacy & Security → **Open Anyway** | Works, but GUI-only and only *after* a block | [Apple 102445](https://support.apple.com/en-us/102445); not reproducible headlessly here |
| Control-click (right-click) → **Open** | **Removed** as of macOS Sequoia 15 | [Apple Developer News](https://developer.apple.com/news/?id=saqachfa) |
| `spctl --add` / `--enable` / `--disable` | **Removed** as of macOS 15.0 | `man spctl` + observed, below |
| "Open Anyway" button *in the dialog* | **Does not exist.** The dialog offers only Move to Trash / Done | Observed, §1 |

### `xattr -d` — the winner

Works, and crucially **works pre-emptively**. The binary does **not** need to be blocked first. Fresh
extraction, attribute removed before the first launch, never prompted:

```
$ mkdir e2 && cd e2 && tar xzf ../montagent-v0.1.0-aarch64-apple-darwin.tar.gz
$ xattr -l montagent
com.apple.provenance:
com.apple.quarantine: 0081;6ab61ad0;;413FE092-…
$ xattr -d com.apple.quarantine montagent      # rc=0
$ xattr -l montagent
com.apple.provenance:
$ ./montagent
montagent ok          # exit 0, elapsed 0s, no dialog ever shown
```

`xattr -d -r com.apple.quarantine <dir>` works the same way on a whole extracted directory
(observed, rc=0, binary then ran). `xattr -c` (clear all attributes) also works but is blunter than
needed.

Per [`man xattr`](#man-pages) on this machine: *"The fourth form, with the `-d` option ("delete"),
causes the given attribute name (and associated value), to be removed."* No privilege note; it
succeeded as an ordinary user on a user-owned file.

### `spctl` — removed, verified on the machine

```
$ spctl --add /bin/ls
This operation is no longer supported. Please see the man page for more information.
```

`man spctl` on macOS 27.0 puts `--add`, `--enable`, `--disable`, `--remove` and `--reset-default`
under a heading literally titled **DEPRECATED OPTIONS**:

> As of MacOS 15.0, operations that modify the rule database or the global state of the assessment
> subsystem will no longer be supported.
>
> To add rules with configuration profiles, please see
> https://developer.apple.com/documentation/devicemanagement/systempolicyrule

So `spctl --add` must **not** appear in the README. The replacement is an MDM configuration profile,
which is not a thing to ask an individual user to do.

`spctl --master-disable` is likewise gone; the surviving `--global-disable` does not disable
anything, it merely *"[r]eveal[s] the option to allow applications downloaded from anywhere in the
Privacy & Security settings pane"* (`man spctl`), requires root, and is a system-wide security
downgrade. It must not be in the README either.

### `spctl --assess` is not a useful check — it rejects even when the binary runs

Line **D** of the probe is a trap worth documenting. *After* quarantine was removed and the binary
ran successfully, `spctl` still says:

```
$ spctl -a -vvv -t exec montagent
montagent: rejected
$ echo $?
3
```

`man spctl` confirms exit 3 means *"an assessment operation results in denial"*. So `spctl --assess`
reports what **would** happen to a *quarantined* copy; it is not a predictor of whether the binary
will execute. **Do not tell users to run `spctl --assess` to check their install** — it will say
`rejected` on a perfectly working one and send them chasing a non-problem.

### Right-click → Open — removed

Apple's developer announcement, in full:

> In macOS Sequoia, users will no longer be able to Control-click to override Gatekeeper when opening
> software that isn't signed correctly or notarized. They'll need to visit System Settings > Privacy
> & Security to review security information for software before allowing it to run.
>
> If you distribute software outside of the Mac App Store, we recommend that you submit your software
> to be notarized. …

— [Updates to runtime protection in macOS Sequoia](https://developer.apple.com/news/?id=saqachfa)

This is why so much existing advice is stale: nearly every "just right-click and choose Open" guide
predates macOS 15.

### System Settings → Open Anyway

Apple's [Safely open apps on your Mac](https://support.apple.com/en-us/102445) gives the flow:
open **System Settings**, go to **Privacy & Security**, scroll down, click **Open Anyway**, then
confirm when the warning reappears.

It works, but it is the wrong instruction for Montagent, for three independent reasons:

1. It is **GUI-only** and **after-the-fact** — it requires the user to have been blocked first, which
   for an MCP subprocess means the failure has already happened invisibly.
2. It is aimed at **apps** the user double-clicks. Our failure mode is a headless subprocess.
3. It is far more steps than `xattr -d`, for the same end state.

It belongs in the README at most as a fallback sentence, not as the primary instruction.

---

## 5. The paragraph the README should carry

Finished prose, ready to paste under the macOS install section:

---

> ### macOS: clear the quarantine flag before first run
>
> Montagent v0.1.0 ships as an unsigned, un-notarized binary, so macOS quarantines it when you
> download the release archive from your browser — and `tar` copies that quarantine onto the
> extracted binary. If you run it without clearing that flag, macOS kills the process with signal 9
> and prints nothing. When an MCP client launches Montagent for you, all you will see is that the
> server failed to start, with no error message to go on.
>
> After extracting, run this once:
>
> ```sh
> xattr -d com.apple.quarantine ./montagent
> ```
>
> That is the whole fix. Do it *before* the first launch and you will never see a dialog. Then move
> the binary wherever you keep it and point your MCP client at it.
>
> If you have already tried to run it, you may have seen a dialog headed **"montagent" Not Opened**,
> offering only **Move to Trash** and **Done**. Choose **Done** — it does not delete anything — and
> then run the `xattr` command above. The same command fixes it after the fact. (Note that later
> launches are killed silently with no dialog at all, so a failure with no visible prompt is still
> almost certainly this.)
>
> Alternatively, download with `curl` instead of a browser and the quarantine flag is never set in
> the first place, so no `xattr` step is needed.
>
> Two pieces of advice you will find elsewhere no longer work: Control-click → **Open** was removed in
> macOS 15 (Sequoia), and `spctl --add` was removed in macOS 15.0. Don't bother with
> `spctl --assess` either — it reports `rejected` for this binary even when it runs perfectly well.
>
> Signing and notarization are planned; until then this step is the cost of a v0.1.0 release.

---

## Sources

### Apple, primary

- [Safely open apps on your Mac](https://support.apple.com/en-us/102445) — Apple Support. The
  *"Apple cannot check … for malicious software"* alert with **Move to Trash** / **Done**; the
  System Settings → Privacy & Security → **Open Anyway** flow.
- [Updates to runtime protection in macOS Sequoia](https://developer.apple.com/news/?id=saqachfa) —
  Apple Developer News. Control-click override removed; System Settings is the replacement;
  notarization recommended. Quoted in full above.
- [Gatekeeper and runtime protection in macOS](https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-sec5599b66df/web)
  — Apple Platform Security. *"Gatekeeper also requests user approval before opening downloaded
  software for the first time."* **Does not** discuss `com.apple.quarantine`, command-line binaries,
  or subprocess execs — the gap this document fills empirically.
- [`LSFileQuarantineEnabled`](https://developer.apple.com/documentation/bundleresources/information-property-list/lsfilequarantineenabled)
  — Apple Developer Documentation. The mechanism by which an app quarantines files it writes.
- [SystemPolicyRule](https://developer.apple.com/documentation/devicemanagement/systempolicyrule) /
  [SystemPolicyControl](https://developer.apple.com/documentation/devicemanagement/systempolicycontrol)
  — the MDM profile replacements `man spctl` names for the removed `spctl` options.
- [Apple Developer Forums thread 683551](https://developer.apple.com/forums/thread/683551) —
  *"Most Unix-y tools don't quarantine their downloads, including curl and scp"*; removing the
  attribute with `xattr`. **[Apple staff on a first-party forum, not formal documentation]**;
  independently reproduced on this machine.

### Man pages (shipped on the test machine, macOS 27.0)

- `man spctl` — the **DEPRECATED OPTIONS** section; the exit-code table (3 = denial); the
  `--global-disable` wording.
- `man xattr` — the `-d`, `-c`, `-r`, `-w` forms.

### Observed on the test machine

All commands, with their real output, are reproduced inline above: `sw_vers`, `uname -a`,
`codesign -dvvv`, `tar --version`, `xattr -w/-p/-l/-d/-c/-r`, `tar xzf`, `spctl -a -t exec`,
`spctl --add`, `spctl --status`, `ps -o pid,ppid,stat,etime,command`, `pgrep -P`, and
`log show --predicate 'eventMessage CONTAINS[c] "montagent"'`.

The test binary was a two-line C program built with `cc`, which produces the same
`adhoc, linker-signed` signature as a default `rustc`/`cargo` release build on Apple silicon.

---

## Not verified

Honest gaps. None of these change the recommendation, but they bound it.

1. **The System Settings row's exact wording.** I could not click through the GUI, so the precise
   label of the transient *"… was blocked …"* row in Privacy & Security on **macOS 27.0** is taken
   from Apple's support page, not read off this screen. Apple's page also **does not state how long
   the blocked item remains listed**, and I could not measure it. If the README ever leans on that
   fallback, someone should read the row on a real screen first.
2. **Headless / no-GUI behaviour.** The decisive question — what happens when there is *no* GUI
   session to show the prompt, e.g. over SSH or in CI — could not be tested: Remote Login is
   disabled on this machine (port 22 closed) and enabling it needs admin rights I did not take. The
   evidence *strongly* implies denial: `syspolicyd` logs `Prompt shown … waiting for response`, and a
   prompt nobody can answer cannot be approved. But whether it blocks forever, times out, or is
   denied outright is **unproven**. This is worth a follow-up ticket if Montagent is ever run in CI
   on macOS.
3. **Whether the dialog itself times out.** Measured block was 108 s, ended by a human clicking
   **Done**. Whether an unanswered dialog would eventually time out, and after how long, was not
   established.
4. **Whether a fresh download prompts again.** The denial is cached (run B: silent kill). I did not
   test whether re-downloading the same binary to a *new path* with a *fresh* quarantine UUID
   re-prompts or is killed immediately by code-hash. This affects how a user experiences an upgrade,
   not the correctness of the `xattr` fix.
5. **Finder / Archive Utility extraction.** I verified propagation through `tar xzf` on the command
   line. Double-clicking the `.tar.gz` in Finder uses Archive Utility, which is quarantine-aware and
   should behave the same or more aggressively — but that path was not tested.
6. **Intel Macs and older macOS.** Everything here is one Apple-silicon machine on macOS 27.0. The
   `spctl` and Control-click removals are documented as macOS 15.0 changes, so macOS 14 and earlier
   will behave differently — more permissively.
7. **Mechanism of libarchive's propagation.** That `bsdtar` propagates quarantine is **observed**,
   with a clean control. I found no Apple documentation stating it, so the *why* is unconfirmed; only
   the *what* is established.
