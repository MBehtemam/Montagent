## Approved font acquisition contract

Source: [approved contract review](https://github.com/MBehtemam/Montagent/issues/952#issuecomment-6113673541), on [the font acquisition map](https://github.com/MBehtemam/Montagent/issues/939).

The map author accepted all final-review recommendations. This contract consolidates the map's resolved decisions and is approved for implementation handoff.

### Operations

CLI `fonts acquire` and MCP `fonts_acquire` share one contract. Input is a discriminated `source`: direct `url`, or `google_fonts` with a family selector or explicit repository file. Google Fonts selectors may include weight/style and a repository revision; resolve a moving revision to one recorded commit before acquiring all related files/materials. Accept optional `face_index` and `expected_sha256`. Caller supplies an acquisition destination; remote calls additionally identify a named workspace. Acquisition never edits project JSON or invokes vendor.

Google Fonts candidate discovery returns exact repository files, revision, inspected family/style/weight/variable metadata and available Faces. Ambiguity returns candidates, never an arbitrary success. A family-only request succeeds only when it identifies a unique file. An exact weight/style requires suitable static bytes; explicitly requesting variable bytes does not select variation axes. Direct-URL selectors are checked against inspected bytes. A collection requires explicit valid Face selection for acquisition success; retain original collection bytes.

CLI/MCP local listing and vendor have parity. Listing identifies execution location. Vendor inputs preserve the existing project/local-file/licence/source/destination contract, with optional local `receipt` and `resolution` references. Vendor alone writes font bytes, evidence and the path-keyed attestation; it returns the chain entry for the agent to paste, never edits Font chains. Broader catalogue/similarity discovery is excluded.

### Structured results and errors

Acquisition returns `status`, `execution`, `candidates` when applicable, `files`, `receipt`, `findings` and `next_step`. Execution identifies local versus remote scope and workspace where applicable. File records include scoped path, SHA-256 and inspected Face metadata. Receipt reference includes its path and SHA-256. Findings distinguish missing accompanying evidence from invalid payload, unsupported source/selection, ambiguous selection, required/invalid Face, expected-hash mismatch, network denial, fetch failure, limit exceeded, invalid destination and collision. These are stable error categories; implementations may choose existing finding-code conventions.

Missing evidence may accompany successful acquisition. Ambiguity or missing Face returns selection-required, not successful acquisition. No incomplete result is published as usable. Existing valid destination bytes survive failed acquisition. URL locators, computed hashes and catalogue membership confer no permission or authenticity.

### Receipt and portable evidence

Receipt schema version 1:

- `schema_version`: 1.
- `source`: kind, requested/final URLs, and Google Fonts commit/path where applicable.
- `retrieved_at`: UTC time.
- `selection`: requested selectors and selected Face index.
- `font`: relative path, SHA-256, inspected file/Face metadata.
- `materials`: original licence/copyright/attribution/notice files, each with relative path, SHA-256, provenance and applicability association to the font hash.
- `licence_findings`: recognized identifiers, missing evidence and material conflicts, distinguished from caller declarations.

Vendor produces a portable evidence manifest referencing the preserved original receipt, original materials and any authorized resolution. The original receipt remains verbatim historical provenance; its historical acquisition paths are not live dependencies. Manifest references are relative to the manifest directory, confined to the project. Each referenced artifact has a SHA-256. The attestation retains `licence`, `source`, `sha256` and adds optional `evidence: {file, sha256}` pointing to that manifest. Existing attestations remain valid. The manifest is evidence, not a second authoritative licence attestation.

All referenced bytes must exist and match when consuming a supplied receipt. Absence explicitly recorded at acquisition remains a finding; later disappearance of recorded materials is an integrity failure. Do not silently discard evidence or retry as receipt-free to evade a conflict/integrity refusal.

### Licence recognition and resolution

Retain the shipped blocklist without override and the existing recognized/declared/unknown gates. Recognized external materials need documented licence-pattern recognition and an applicability association to exact font bytes. Initial automatic association: official Google Fonts family licence/notice files and selected font file from one pinned repository revision, identified by repository-relative paths. A nearby licence filename or a direct URL alone is insufficient. For other sources without recognized applicable metadata, retain evidence and require explicit caller declaration rather than infer authority.

The author accepted this initial association rule; new disputed provider/legal facts require primary-source research.

Material conflicts stop vendor. Aliases, harmless wording differences and compatible alternative licensing alone do not. The resolution record contains selected applicable licence, rationale, conflicting evidence references/hashes, font hash, caller adoption and authorization basis. Agents may draft or relay it using existing explicit authorization, but cannot invent caller authority. The tool records a caller assertion, not a legal verdict or authenticated identity guarantee. Existing recognized metadata versus explicit declaration conflict rules remain effective.

### Local example

1. Discover Google Fonts candidates and select an exact static file at a pinned commit.
2. Invoke acquisition with destination and optional expected hash; inspect returned receipt/findings.
3. Invoke vendor with project, local font path and receipt; supply caller declaration/resolution only when required.
4. Apply returned attestation/chain data through normal agent authoring. Preserve chain order and Face identity.
5. Move the complete project directory; all declared chain files, including fallbacks, their required materials and attestations remain usable offline. Unused acquisition files are not dependencies; no pruning or packaging command is added.

### Remote example and recovery

1. Select a named workspace and invoke remote acquisition, or explicitly upload locally acquired bytes/evidence through the authenticated HTTP byte door.
2. Invoke vendor against those workspace files and the remote working copy.
3. Retrieve the portable bundle through the byte door, verify hashes and reconcile project-relative paths before updating authoritative agent disk/Git/workflow storage. No font base64 in MCP.
4. Reuse verified bytes. After workspace loss explicitly upload or reacquire required exact bytes; changed/missing bytes cannot silently substitute a newer version.

Bound fetch size, duration and redirects, checking network policy at every redirect. Enforce workspace path confinement and existing collision rules. No fetching during font validation/rendering; remote media-source fetching is a separate existing exception. Remote transport is accepted design, not shipped behavior in this checkout.

### Acceptance criteria

- CLI/MCP equivalent inputs produce equivalent semantic results; remote paths are unmistakably scoped.
- Ambiguous family, selector conflict, unsupported variable instance and unselected multi-Face collection cannot report acquisition success.
- Native TTF/OTF/TTC/OTC bytes are preserved; WOFF/WOFF2, archives and HTML are rejected, without conversion/extraction.
- Expected-hash mismatch or interrupted fetching preserves existing bytes and publishes no successful receipt.
- Absent accompanying evidence is distinguishable from changed referenced evidence.
- Vendor checks blocklist first, rejects conflicts/integrity failures, and never substitutes or edits Font chains.
- Supplied evidence and resolution survive project relocation and loss of the acquisition workspace.
- Existing attestations remain valid; multiple chains referencing one path share one attestation.
- Offline validate/render use declared vendored bytes without fetching or live licence adjudication.
- Remote cleanup recovery is explicit and hash-verified; unsupported providers give actionable external-acquisition instructions.

This planning effort approves the specification only. No implementation was performed.


