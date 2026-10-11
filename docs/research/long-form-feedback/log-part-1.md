# Log: Part 1 of the whole tale (2026-10-10 to 2026-10-11)

Part 1 = chapters 1-4 of Andersen's "Den grimme Ælling" (64 units, 720 Danish words), 49 min 17 s, 1920x1080, 490 MB. Files: `out/part1-upload/` (video, thumbnail, metadata). Data: `part1/`. Builders: `build_part.py`, `post_part.py`. Companion notes: `findings.md` (items 50-60), `log.md`, `profile-2026-10-09.md`, `source/CHANGES.md`.

## 1. Result in numbers
| | |
|---|---|
| Video | 2,957.4 s, 88,722 frames, 30 fps, h264 + aac, loudness -16.0 LUFS (true peak -2.8 dB) |
| Content | 64 units, 19 spreads, 6 scenes (pictures), 23 stickers (17 new), 1,060 marks, 1,386 tracks |
| Audio | 64 Danish + 66 English lines (Azure), 32.7 min of speech before gaps (43 units) |
| Render | 10 chunks, 3.24 h of render clock, 7.60 fps average = 0.253x realtime (+ 8.5 h lost to a sleeping laptop) |
| Density measured | **4.1 s of video per Danish word** at full teaching |
| Whole tale (estimate) | about 3.3 h at this density, 2-2.5 h with the taper, about 12 h of rendering, about 3 parts |

## 2. What went well
1. **Decisions were made with the user before work.** The original-wording choice, the tapering, "keep pauses and extras", and part length were settled early, so there was no rework on the text plan.
2. **Licence checked up front** (author died 1875; own spelling modernisation, own English, own pictures; credit line in the description).
3. **The source text was diffed against the 1843 original** (token level). It found wording changes I had not disclosed (aflægge besøg, snakke, klodset, ...) and two typos; all are listed in `source/CHANGES.md`.
4. **Two review agents before audio** (script checker, learner) found real mistakes: "blomme" is a plum, "always second place" is too strong, stems lose their e (snakk+ede), sin/sine only with one owner, a leftover "last time we saw", dense units (c01-04, c01-10, c01-13). Fixing them before the audio was rendered saved a re-record of most of the guide.
5. **Marks written by word text, not by index** (`part1/marks.py`). A word that the guide never says is now a hard build error, which found a dozen wrong triggers immediately instead of mis-timed marks in the video.
6. **Scene-based pictures**: 6 pictures served 64 units (spreads re-filled by real text height). Cost and review effort fell by about 5x compared with one picture per page.
7. **Chunked render + `--progress-file`** worked as hoped: live percent and ETA, a notification per chunk, a failure costs one chunk, resumable. The join is exact (88,722 frames) and the seams are invisible and silent.
8. **Checking real frames caught what validate cannot:** the cover label on the plaque border, the end-card Like pill covering the channel name, touching glosses, rows running to the page edge, and a stray tripod in a sticker loop.
9. **Measurements replaced guesses** where it counted (1080p speed, density per word, loudness).
10. **The user's corrections turned into durable rules** (CLAUDE.md working rules, `references/user-requirements.md`, memory notes) the same day.

## 3. What did not go well (and what it cost)
| # | Problem | Cause | Cost / fix |
|---|---|---|---|
| 1 | **Wrong estimate of the video length** (I said 9 s per word, about 4.7 h and 5 parts). Real: 4.1 s per word, about 3 h, 3 parts. | I extrapolated from the pilot's short sentences. | The intro said "five parts" and had to be re-recorded; the plan was reported wrongly to the user for several messages. Measure the audio first. |
| 2 | **Part 1 first built as chapters 1-3: only 35 min**, not the hour the user wanted. | Same wrong density. | Chapter 4 added afterwards (21 units of guide, marks, 2 scenes, new audio, outro rewritten, c03-15 re-recorded). |
| 3 | **I paraphrased the user's requirement** ("small loop animation ... not to distract" became "move only slightly") and then explained the drift problem on that wrong basis. | Summarising instead of quoting. | User correction; docs and memory rule written. The true requirement is *slow* animation; the drift is a defect. |
| 4 | **Said Veo "cannot" be fixed** without having tested the one option (first frame = last frame). | Overconfident wording. | Corrected to "measured / not tested"; rule added. |
| 5 | **Thumbnail rejected three times** (quiet book cover; a stack of tidy variants; a "pretty" open book with small title). Also I said `thumbnail.png` was the new design while the file was still the old one. | I designed for how it looks large, not for a stranger at 360 px. Did not check the file I had claimed. | Final: bands + big Impact text + real Danish and English line, tested at 360 px. Rule written: a thumbnail must engage a stranger. |
| 6 | **The laptop slept in the middle of the 1080p render: 8.5 h lost** (render clock 1,250 s vs wall clock 9 h). | No `caffeinate`. | Nothing lost, but a night wasted. `caffeinate` is now started by `render_chunks.py run`. |
| 7 | **`sc()` clamps negatives to 0** (a bug since the pilot builder): any gloss pushed *left* to avoid another gloss was never moved, so neighbours touched ("keep yourselvesalways"). | Helper written for sizes, reused for offsets. Found only by looking at frames. | Fixed with a plain rounded multiply; also gloss width x1.25 and gap 28 px. |
| 8 | **Sticker generation chaos:** zsh does not split `$VAR` (first run died with a KeyError), 429 errors even with one worker, then two chains ran at the same time and fought each other. | Shell habit; no retry in `gen_stickers.py`. | About 25 minutes lost; retry with 20 s backoff added. Veo loops were not duplicated. |
| 9 | **Veo drew a tripod** twice (a scene, then a sticker) because the prompt said "tripod". | Prompt wording. | Wording changed to "the camera does not move" everywhere. |
| 10 | **My own `pkill` killed three watchers** I had armed. | Careless cleanup. | Re-armed; noise only. |
| 11 | **Render ETAs I gave were wrong three times** (3.8 h, 5.2 h, 4.5 h; actual 3.2 h). | The ETA follows the current fps, which depends on the part of the film (intro 5.8 fps, pages 7.9 fps); the 30 s slice test was 29% too pessimistic. | Rule: estimate = sum over chunks, and say it will move. |
| 12 | **End card first built with positions in fractions of the width but text sizes from the height**, so on the larger card the Like pill covered the channel name. | One layout formula reused at two aspect ratios. | End card re-proportioned (bar 1590x150). |
| 13 | **Slow builds:** `build_part.py` takes 63 s because of many `montagent measure` calls (one per text). | A subprocess per measurement. | Not fixed. A font-metrics cache (or a batch `measure`) would cut it to seconds. |
| 14 | **Five phrase boxes are skipped** because the phrase wraps over two lines; five "order warnings" fire for events listed out of speaking order. | Boxes are one rectangle; marks are authored by hand. | Accepted for Part 1 (glosses still show). Better: draw one box per line. |
| 15 | **R-HIGHLIGHT-UNPAINTED** on one 17 ms word (c02-08 "er"): that karaoke word is never drawn. | Very short word at 30 fps. | Harmless; reported. |

## 4. Issues found by reading frames (all fixed unless noted)
- Cover: "Første del" sat on the plaque border -> moved under the author.
- Page text: row under the picture ran to the seam (c04-18) -> rule rows shortened; the builder now warns when a row is wider than 550 px or there are more than 5 rows.
- Key lines for long phrases ("fra muren og ned til vandet = ...") were 840 px wide -> key shows the first three words + "...".
- Gloss pairs "was to" / "to hatch" collided -> shorter gloss + the push fix above.
- Page-turn crossfade shows both spreads ghosted for 0.6 s (by design; looks busy when both are crowded). Not changed.
- Pictures are small on the page (740 px wide on a page 1,000 px tall): calm, but there is empty space; a taller crop of some scenes would use it better.

## 5. Open items (not done)
1. **A native speaker has not checked** the Danish wording or the voice: ked af det (Andersen: weary; today: sad), Dyrehavstiden, deres vs sin, kalkun-æg vs kalkunæg, om jeg så selv må (c04-07).
2. **I have not listened to the audio** or watched the video from start to finish; only frames and numbers were checked. The English claims in the 21 chapter-4 guide lines were written after the review agents ran, so they were **not reviewed** (run both agents on c04 before Part 2).
3. The slower, calmer loop speed (`SLOW` 2.0) was offered, not applied.
4. Part 2 must exist before the "Next: Part 2" end card is true.
5. YouTube upload not done (file is far over the 10 MB browser limit; the user uploads it).
6. B variant (Jeppe + Davis) not built.

## 6. What would help next time
**Process**
- Measure first: render the audio of a few units and count seconds per word before promising a length.
- Quote the user's words (see `references/user-requirements.md`) in every plan and every explanation of a design.
- Run the two reviewers on *all* guide text before audio, including late additions.
- Look at frames after every layout change; add a width/overlap check for anything drawn from estimated text widths.
- Run long jobs under `caffeinate`, with one watcher per job; never `pkill` by pattern while watchers are armed.
- Make 2-3 thumbnail variants and test each at 360 px before showing them.

**Tools (ours)**
- Cache `measure` results on disk (build 63 s -> seconds).
- One phrase box per line; overlap check for glosses done in the builder, with a printout.
- Stabiliser without PNG files on disk (stream through memory) and with real feature tracking; delete frames after baking (`_work` is 1.6 GB).
- `gen_stickers.py` / `gen_sticker_loops.py` with retries and a single queue so two runs cannot collide.

**Montagent (asks)**
- A way to cache static layers (table + book) between frames: they never change and the 1080p render is 3.6x slower than the 540p draft.
- A `paused` / wall-clock-gap field in the progress file; windowed `fps`; a count of drawn elements to explain slow stretches.
- A per-project switch for noise reviews (`R-KEYFRAME-UNREACHED` 1,470; `R-EASE-INERT` 1,163).
- Batch `measure`; a group/parent transform (1,060 marks are 1,060 tracks).
- A stabilise / "lock to frame 0" effect for video clips, so drift is not fixed outside the tool.

## 7. Time line (local time)
- Evening 10 Oct: user starts the whole-tale request; text sourced and modernised; guide written and reviewed; 4 scenes; audio; builder generalised; chapter 4 added.
- 10 Oct night: stickers, branding (Like pill, handle), checks of frames; render started 18:48 (30 s test slice earlier: 5.9 fps).
- 10-11 Oct: render stalled while the Mac slept (about 8.5 h), then ran 2 h 45 min to the end of chunk 10; join, loudness, upload folder at 05:41.

## 8. Profile: how long each step took
Sources: measured with timers during the run (marked **M**), file timestamps of the outputs (marked **T**, system clock UTC+2, so waiting and writing are mixed in), or read from the render's progress files (**P**). Times from the 30 s test slice are on the pilot, not on Part 1.

### 8.1 Compute and service time (what machines and APIs spent)
| Step | Amount | Time | How |
|---|---|---|---|
| Download Andersen's text (Wikisource API) | 19 KB | under 1 s | M |
| Nano Banana 2.1, 6 scene plates (1376x768) | 6 | 12.1 / 12.5 / 13.1 / 18.5 / 16.7 / 17.9 s (one at a time, no 429 when sequential) | M |
| Veo 3.1 Lite, 6 scene loops (4 s, 720p) | 6 (+1 rerun without "tripod") | 4 submitted together: 94 s for all; 2 submitted together: 50 s; rerun 40 s | M |
| Stabilise + bake a scene loop (96 frames) | 6 | about 6 s each | M |
| Nano Banana for 17 sticker stills | 17 | about 12-16 s each when it passes; the batch took about 25 min of wall time because of 429 answers, 20 s back-offs and one duplicate run | M, T |
| Veo sticker loops | 17 (+1 rerun) | 35-37 s each, 3 at a time: 244 s for all 17; rerun 42 s | M |
| Bake sticker loops (ffmpeg) | 17 | under 1 s each | M |
| Azure TTS Danish | 64 lines, about 4,000 characters | about 2 min | T, M |
| Azure TTS English | 66 lines, about 21,000 characters | first 45 lines over 120 s (timed out into the background), next 21 lines about 100 s, one line 5 s | M |
| `build_part.py` (design project) | 64 units, 1,060 marks | **63 s** (almost all of it `montagent measure`, one call per text) | M |
| `post_part.py` (book scale, stickers, branding) | 23 stickers | few seconds | M |
| `montagent validate` | 1,386 tracks, 1.7 MB JSON | **3 s** | M |
| 30 s render slice at 1080p (pilot) | 900 frames | 152.9 s = 5.9 fps | M |
| **Render, 10 chunks, 1080p** | 88,722 frames | **3.24 h** render clock = 7.60 fps average (chunk 1: 5.78 fps; chunks 2-10: 7.5-8.1 fps) | P |
| Join (ffmpeg concat, stream copy) | 10 files | a few seconds | M |
| Seam check (9 joins: frames + audio) | | about 20 s | M |
| Loudness: measure + linear gain + AAC encode | 49 min | about 3 min | M |
| Thumbnails | 5 designs | each under 3 s (PIL) | M |

### 8.2 What the render cost, and why
- **Elements by type (final project):** 728 text, 273 rect, 188 path, 150 ellipse, 130 audio, 29 video, 17 image. **My first reading of this was wrong:** I said that, because about 80% of the elements are small marks and glosses, the render is dominated by drawing text and shapes. The profile in section 12 shows the opposite: the two full-frame images (table and book plate with its feathered edge) cost about 75% of every frame, and marks, glosses and text together cost about 10-15%.
- **Speed by part of the film:** intro (cover swivel with perspective, several fades, name bar) 5.8 fps; ordinary book pages 7.5-8.1 fps. The 30 s test slice (5.9 fps) came from a dense mid-book stretch of the *pilot* and over-estimated the cost by 29%.
- **1080p vs draft:** the pilot's 540p draft ran at 0.70x realtime; Part 1 at 1080p ran at 0.253x. About 2.8x slower, not the 3.6x the slice suggested.
- **Estimates I gave while it ran:** 3.8 h, then 5.2 h (first chunk's low fps), then 4.5 h (after the sleep), then 3.4 h; actual 3.24 h. An ETA from one chunk is not reliable; sum over chunks.
- **The sleep:** chunk 1 took 32,343 s of wall time but only 1,446 s of render clock. 8.5 h lost, nothing re-rendered.

### 8.3 Where the working time went (from file times, UTC+2; includes thinking and waiting)
| From - to | Phase |
|---|---|
| 16:39 - 16:46 | source text fetched; 293 units modernised and written (3,363 words) |
| 16:47 - 16:56 | 4 scene plates, 4 Veo loops, stabilise and bake |
| about 16:50 - 17:00 | guide script for chapters 1-3 written (about 3,500 words); two review agents run in parallel (about 3 min) |
| 17:00 - 17:32 | review fixes applied; Danish + English audio generated; builder `build_part.py` assembled and debugged; the first build (chapters 1-3 only: 35 min of video) |
| 17:00 - 17:12 | chapter 4 added: scenes s5, s6 (plates, loops, tripod rerun), 21 more units of guide and marks |
| 17:18 - 17:39 | 17 new stickers (stills, loops, bake); like.png drawn |
| 17:30 - 17:47 | marks fixes (rows too wide, gloss collisions), end-card re-proportioned, `sc()` bug found and fixed |
| 17:48 | final project built and validated, chunked render started |
| 17:48 - 05:38 (next day) | rendering (3.24 h) and sleeping (8.5 h) |
| 05:38 - 05:41 | join, checks, loudness, upload folder |
Hands-on time from the first download to the start of the render: **about 70 minutes of wall time**. Waiting on the render: 3.2 h of compute; with the sleep, 11 h 50 min of wall time.

### 8.4 Disk
| | Size |
|---|---|
| `assets/` (plates, loops, stickers, brand) | 345 MB (stickers 161 MB, 35 loops) |
| `audio/part1/` | 33 MB |
| `out/chunks/` (10 chunk files; safe to delete after the join) | 463 MB |
| `out/part1-upload/` (final video 490 MB + thumbnails + metadata) | 487 MB |
| `_work/` (stabiliser frames, test frames, old projects; all regenerable) | 1.6 GB |
| `project-part1-A.json` (the Montagent project) | 1.7 MB |
| `part1/` (the part's data) | 112 KB |

### 8.5 Rules of thumb that came out of this
- **Video length:** 4.1 s per Danish word at full teaching, from the measured audio (not from the word count of sample sentences).
- **Rendering at 1080p:** plan **3.9 h of rendering per hour of video** for this style (Part 1: 0.82 h of video took 3.24 h; book pages are a little cheaper, the intro dearer). The whole tale (about 3 h of video) is then about 12 h of rendering. (Earlier notes said 1.4 h per hour; that figure was wrong and is corrected in `findings.md` item 59.)
- **Build + validate** are cheap (about 1 min); **only the render is expensive**, so look at frames before every render.
- **Cost of a new picture:** about 15 s to generate, 40-95 s to animate, 6 s to stabilise; 17 stickers cost about 5 minutes of API time if nothing collides.

## 9. CPU use and parallel rendering (measured afterwards, 2026-10-11)
Machine: Apple M1 Pro, 10 cores (8 performance + 2 efficiency), 16 GB. Test: 12 s slices (360 frames) of the final project at 1080p, mid-book (from 1,500 s), sampling `ps` every second for `montagent` + `ffmpeg`.
| Test | Result |
|---|---|
| **One render** | mean **118% CPU** (max 131%; 100% = one core), about **1.2 of 10 cores**; memory about 610 MB; 360 frames in about 45 s (about 8 fps, the same as the chunks) |
| **Four renders at the same time** (different ranges) | 52, 52, 53 and 54 s each (about 15% slower each); mean 460% CPU (max 511%), memory 2.4 GB; **aggregate about 26.7 fps = 3.3x the single render** |
- **Conclusion: the render is almost single-threaded** and leaves about 88% of this Mac idle. Montagent has no thread option (`render --help` lists none), but chunks are independent files, so running chunks side by side is safe and scales.
- **What it would have meant:** Part 1 at 4 parallel jobs: 88,722 frames / 26.7 fps = **about 55 min instead of 3.24 h**. The whole tale (about 12 h serial) would be about 3.6 h. 6-8 jobs were not tested; memory (about 0.6 GB each) would allow 8, but only 8 performance cores exist, so I expect 5-6x, not 8x, and the disk and ffmpeg encodes may become the limit. Test before relying on it.
- **Cost:** none in quality (each chunk is a self-contained range render, already joined without seams). Slight risk: all four jobs finish together, so a failure wastes more; and the Mac is busy (fans) while it runs.
- **Next step (not done, needs the user's go-ahead):** `tools/render_chunks.py run --jobs 4` with `caffeinate`; plan chunks so that their number is a multiple of the jobs and their lengths are similar (the intro chunk is slower).
- **Also measured:** the 1080p slowdown vs the draft (2.8x) is a per-frame cost, not a lack of cores; a 540p draft would also run in parallel.

## 10. If we ran Montagent on GCP or Azure (hosted): what is needed for its full potential
Status: **a plan, nothing built or tested in the cloud.** Numbers marked (M) were measured on the M1 Pro; (E) are my estimates and must be checked with a calibration run. I did not look up cloud prices (they change); the cost formula is given instead.

### 10.1 What the workload looks like (M)
| | Part 1 | Whole tale (about 3 parts) |
|---|---|---|
| Frames at 1080p | 88,722 | about 270,000 |
| Render work | one render uses about 1.2 cores and about 0.6 GB; 3.24 h at about 8 fps, so **about 3.9 core-hours** | **about 12-15 core-hours** (M1 Pro speed) |
| Per-chunk input | the project JSON (1.7 MB) + `assets/` 345 MB + `audio/` 33 MB + fonts | shared assets (table, book plates, brand, fonts) are the same for every part |
| Output | chunks 463 MB, final 490 MB | about 1.5 GB |
- Cores, not memory, decide the speed: four jobs in parallel were only about 15% slower each (3.3x aggregate). More jobs should keep scaling until the cores run out.
- Not measured: whether Montagent uses a GPU (CPU use alone was about 1.2 cores), how a cloud vCPU compares with an M1 Pro core, and the speed of ARM versus x86 instances.

### 10.2 What we need to have (the checklist)
1. **A Linux build of the Montagent CLI** (x86-64 and, ideally, ARM64) with its licence/terms for running on servers; ask the Montagent owner whether a hosted/headless mode exists. Without it nothing else matters.
2. **A container image**, version-pinned: Montagent 0.2.x, ffmpeg (join + loudness), Python 3 with Pillow, the story fonts (`montagent fonts vendor` already puts them in the project folder). Same image for every job.
3. **Object storage as the single source of truth** (GCS bucket / Azure Blob container): `projects/<story>/<part>/` with the project folder, `audio/`, `assets/`; `chunks/` for outputs; `final/` for the joined file. Asset paths in the project are relative, so a job only has to copy the folder to a local disk and run.
4. **A job runner that starts one job per chunk** (a chunk is already `--from/--to` with its own `--progress-file`): GCP Batch / Cloud Run Jobs / GKE jobs, or Azure Batch / Container Apps jobs / AKS. A job = pull project (cached), render, upload the chunk, exit. Idempotent: if the chunk already exists in storage, skip.
5. **A coordinator** (our `render_chunks.py` extended): plan chunks, submit jobs, wait, join with ffmpeg, loudness pass, write the upload folder. Join and loudnorm take about 3 minutes on one core.
6. **Progress and notifications:** each job writes its `--progress-file` to storage (or a log line) every few seconds; one small poller sums frames done and prints the same overall percent and ETA we use now; send a message per finished chunk (the Monitor we use today).
7. **Instance choice by calibration, not by guess:** run the 12 s test slice at 1, 2, 4, 8 and 12 parallel jobs on two or three machine types (x86 16 vCPU, an ARM 16 vCPU type, a bigger one) and record fps per job. Expect about one job per 1.2-1.5 vCPU and 0.6-1 GB per job (E).
8. **Spot / preemptible machines** are a good fit because the unit of retry is one chunk (5 min of video), not the whole film.
9. **Cost estimate before every launch** (the repo rule "report what a run will do before a bulk job"): jobs x chunk minutes, core-hours = frames / (fps per job) / 3600 x cores per job, price = core-hours x the instance's current hourly price per core. Part 1 is about 4-6 core-hours (E, cloud cores may be slower), the whole tale about 15-25 (E). Add a budget alert and make every VM delete itself at the end.
10. **Same machine type for all chunks of one video.** Different CPU architectures can give slightly different pixels (font rasterising, floating point). Our seams are invisible today because one machine rendered all chunks; check the seam difference (we measured 0.5/255) when chunks come from different types.
11. **Credentials and quotas:** the generation steps (Nano Banana, Veo, Azure Speech) already run in the cloud, but from the laptop with a personal token. In a pipeline use a service account / managed identity with minimal rights, keys in Secret Manager / Key Vault, never printed. Raise the image-generation quota (we got HTTP 429 even with one worker); Veo and Speech concurrency limits need checking before a batch.
12. **Egress:** only the final 0.5 GB per part leaves the cloud; keep chunk files in the bucket and delete them after the join.

### 10.3 Expected gain (E, to be checked)
- At the same per-core speed as the M1 Pro: 16 vCPU = about 10-12 jobs = 80-95 fps aggregate: **Part 1 in about 15-20 minutes** (the intro chunk is slower and is the last to finish). A cloud core about 40% slower: about 25-30 minutes. Both against 3.24 h on the laptop.
- Several parts at once (Part 2 and 3 rendered while Part 1 is checked) is possible because the jobs share nothing.

### 10.4 What would let Montagent itself use the hardware (asks to the Montagent owner)
- A built-in `--jobs N` (or a chunk scheduler) for one project; a thread-count option.
- Cacheable static layers (table + book plate never change): every frame redraws them today.
- A GPU path, if one exists; a headless/container-friendly Linux build.
- Progress reporting to a URL or stdout in a machine form, plus a "paused / stalled" hint (the sleep incident would have been visible).
- A content-addressed asset cache, so a worker does not copy 345 MB for every chunk.

### 10.5 Order in which I would do it
1. Add `render_chunks.py run --jobs 4` and measure on this Mac (cheap, no cloud).
2. Ask about the Linux build and licence for hosted use.
3. Build the container, run the calibration slice on 2-3 instance types, decide.
4. Run Part 2 in the cloud as the first real test, with Part 1's local render as the reference for seams and speed.

## 11. Can this Mac's GPU be used? (checked 2026-10-11)
Short answer: **not by Montagent as it is today, and it would not be the main lever anyway.**
| Check | Result |
|---|---|
| Binary | `montagent` is an arm64 Mach-O (native on Apple Silicon); `otool -L` lists **no Metal, VideoToolbox or other GPU framework** |
| Encoder | hard-wired **libx264, preset medium** (the strings say "ffmpeg 7.1 or newer, built with libx264"; our output reports `Lavc63.1.102 libx264`); no encoder, codec, preset, thread or GPU option in `render --help` |
| GPU load during a render | `ioreg` (IOAccelerator) over a 8 s slice: **renderer utilisation mean 0.3%, max 8%**; device utilisation 7% (the desktop's own, 11% before the render started). The GPU is idle |
| What the binary contains | strings for `webgpu`, `vulkan`, `metal`, `sksl_gpu` (these look like the drawing library's optional GPU back-ends, compiled in but not used); I could not see how to switch them on, and there is no flag |
| What the Mac offers | ffmpeg here has `h264_videotoolbox`, `hevc_videotoolbox`, `prores_videotoolbox` (the hardware video encoder); the M1 Pro has a 16-core GPU |
- **Where the time goes (E, from the CPU numbers):** one render uses about 1.2 cores: about one core for compositing the frame (text, shapes, video, fades) plus about 0.2 core for libx264. So the encoder is only about 15% of the work; **a hardware encoder would not speed the render up much**, and Montagent would have to expose it anyway. The compositing (about 1 core, single-threaded) is the bottleneck. **What it spends its time on is in section 12:** not text and shapes, as first written here, but mostly the two full-frame background images and the feathered mask.
- **Could the compositing run on the GPU?** Only if Montagent turned on a GPU drawing back-end. That is a Montagent change (ask the owner: is a Metal/WebGPU raster path planned? does it keep identical pixels?). Not something we can do from the project file.
- **Where a hardware encoder would help:** our own ffmpeg steps (not the Montagent render): the join is a stream copy (no encode); the loudness pass re-encodes only audio; a future re-encode of the video for a smaller upload file could use `h264_videotoolbox` (fast, but larger/lower quality per bit than libx264; check before using for the final upload).
- **What does help on this Mac:** the 8 idle cores: parallel chunks (section 9), about 3.3x with four jobs.
- **In the cloud (section 10):** a GPU VM would not help either unless Montagent gets a GPU path; CPU instances with many cores are the right purchase.

## 12. Where the time goes inside one frame, and how to make it faster (measured 2026-10-11)
Method: a 3 s slice (90 frames) of the final project at 1500 s (open book, picture loop, 2 stickers, 29 active elements, 7.2 fps = 0.138 s/frame), rendered once with the whole project and then with one kind of element removed or edited (`tools/mrender.py profile` for the coarse run; a finer script by element id for the rest). Savings are not additive.
| Variant (same slice) | s/frame | Saves |
|---|---|---|
| everything | 0.1383 | - |
| drop the table plate | 0.1023 | 26% |
| drop the book plate (`spread-full`, feathered mask) | 0.0624 | 55% |
| **drop table + book plate** | 0.0346 | **75%** |
| drop the picture loop only | 0.1239 | 10% |
| drop stickers only | 0.1342 | 3% |
| **drop picture loop + stickers (both)** | 0.0328 | **76%** |
| picture loop without multiply blend / without blend and masks | 0.1325 / 0.1320 | 4% / 4.5% |
| drop the full-frame fade rectangle | 0.1333 | 3.6% |
| drop all marks, glosses, arrows, key | 0.1253 | 9% |
| drop the karaoke text blocks | 0.1325 | 4% |
| strip every effect (masks, shadows) | 0.0884 | 36% |
| strip every blend | 0.1276 | 8% |
Coarse run (all active elements of a type): without image 64%, without video 62.5%, without text 20%, without rect 11%, without path 8%, without ellipse 4%, without effects+blend 33%.

**Reading it**
1. **Two full-frame images cost three quarters of every frame** (1935x1080 table with a 'cover' fit and a clip; 1800x1005 book plate with a 26 px feathered mask). They never change, yet they are drawn again for every frame.
2. **Any video element in the project switches the saving off (found later the same day, see 12b).** Removing the picture loop alone looked like 10% in the full project and like 78% in a reduced copy, because the full project still contained other video elements (other scenes, stickers) elsewhere in the film. A decisive test showed that one small video element placed 500 s away from the rendered slice makes every frame about 4x slower; a small image there changes nothing. So with any video in the project, every frame is drawn from scratch and the two big images are paid for every time; my first explanation (two videos together) was a half-truth.
3. **Effects (masks, shadows) cost 36%**, mostly the feathered book-plate mask; blends 8%.
4. **Text, shapes and marks are the small part** (all marks 9%, karaoke text 4%, shapes less). The idea that the many marks make the render slow was wrong.

**Test of a fix we can do ourselves (nothing changed in the project; temp files only)**
Bake the table and the open book (with the feather) into one 1920x1080 picture, using `montagent frame --full` on a project that contains only those two elements, and use it as the single background after the book has opened.
| Same slice | time for 90 frames | speed |
|---|---|---|
| as built (table + book plate + mask) | 14.5 s | 6.2 fps |
| one baked background picture | 2.7-2.9 s | **31-33 fps, about 5x faster** |
Picture difference between the two outputs (frame at 1.5 s): mean 1.5/255, no pixel off by more than 24 levels (re-encoding noise), so it is visually identical. (A first attempt wrote all runs to one output file and gave nonsense timings; each run needs its own output path.)
- **Expected effect (E):** Part 1 from 3.2 h to roughly 0.7-1 h in one process (the opening 35 s and any slice with brand cards keep their own cost); with 4 parallel jobs (section 9) about 15-20 min. The whole tale, from about 12 h serial to about 3 h, or well under an hour in parallel. Only the slice at 1500 s was tested; a full render is needed to confirm.
- **How to apply it:** in `post_part.py`: after the opening, replace `table` + `spread-full` by one `bg` image (baked once per project), keep the original elements for 0-35 s (cover swivel). Stage as an option and compare one full chunk against the old one before trusting it.
- **More on our side:** pre-scale the table plate to the frame size (it is stretched 1.4x per frame); replace the full-frame black fade rectangle by a fade only at the start and end; fewer or lighter shadows on stickers.

**What we need from Montagent (ranked by the evidence above)**
1. **Cache static layers across frames** (an element or group that does not change between frames, here the 2 plates): proven worth 5x here. Ideally automatic, with a way to see which layers were cached.
2. **Keep the static-layer fast path when a small video is active** (point 2 above): re-draw only the regions that change (dirty rectangles): the picture, the karaoke line, a sticker.
3. **A cheaper feathered mask**: compute the mask once per element, not per frame (effects are 36% of the time).
4. **A `cache`/`bake` option on an element or group** ("this layer is static, render it once"), and group/parent transforms (1,060 marks are 1,060 tracks).
5. **A per-element render-time report** (`render --profile`, time per element id/type): this analysis needed 14 renders.
6. **Parallel frames inside one render** (`--jobs N` / threads) and compositing and encoding on separate threads: today one render uses 1.2 of 10 cores.
7. **A GPU drawing back-end** (lowest priority now: the cost is redundant work, not raw drawing speed).
8. Make the build side faster too: batch `measure` (the 63 s part-builder time).

### 12b. The decisive test (2026-10-11, reduced copy of the project = `montagent-repro/`)
| Variant of the 3 s slice | s/frame |
|---|---|
| everything (picture loop present) | 0.129-0.134 |
| picture loop removed (no video element left in the project) | 0.026-0.031 |
| same, plus ONE small video element at 2,000 s (outside the rendered range 1,500-1,503 s) | **0.120-0.125** |
| same, plus ONE small image element at 2,000 s (control) | 0.027 |
| full project (1,386 tracks, many video elements elsewhere), picture loop removed | 0.126 (still slow) |
| full project, audio elements removed | 0.124 (no change: audio is not the cause) |
| full project, only elements active in the slice (= the kit), picture loop removed | 0.027-0.029 |
Reading: the presence of a video element anywhere in the project makes every frame slow, whether or not it is on screen in the rendered range. Not confirmed with the Montagent developer; reproduce with `montagent-repro/video_presence_test.py`.
Consequence for us: our films always contain video elements (picture loops, stickers), so we always pay for the two big background images in every frame. A baked background (one image) removes most of that cost even with video present (0.023 s/frame in the kit).
Developer package: `montagent-feedback.md` and `montagent-feedback.json` (user stories and acceptance criteria), `montagent-repro/` (reproduction kit).
