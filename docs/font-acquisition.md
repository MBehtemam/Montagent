# Explicit font acquisition

Acquisition downloads native TTF, OTF, TTC or OTC bytes and records a receipt. Vendoring is a separate, local-file operation: it checks the licence gate, copies bytes and evidence into the project, writes the path-keyed `fontVendor` attestation, and returns a Font chain entry for the agent to paste. Neither operation edits Font chains. Validation and rendering never acquire missing fonts.

## CLI

```sh
montagent fonts list --json
montagent fonts acquire ./acquired --google-family "Inter" --json
montagent fonts acquire ./acquired --google-file 'ofl/inter/Inter[opsz,wght].ttf' --revision <commit> --json
montagent fonts acquire ./acquired --url https://example.org/font.ttf --expected-sha256 <sha256> --json
montagent fonts vendor project.montagent.json ./acquired/font.ttf --receipt ./acquired/receipt.json --json
```

Use the returned file and receipt paths, rather than assuming the example basenames. Acquisition always returns structured JSON; `--json` is accepted for consistency. Choose exactly one source flag. Google selectors accept `--weight`, `--style` and `--revision`. A family that matches several files returns selection candidates, never a guessed selection. Pin an exact file and revision when reproducibility matters. Exact weight/style selectors require suitable static bytes; selecting a variable file does not select variation axes. Collections require an explicit valid `--face-index` and preserve the original collection bytes. Vendor accepts the same `--face-index`.

`--receipt` verifies the font and every material claimed present before preserving a portable evidence bundle. `--licence` supplies an explicit caller declaration when recognition is insufficient. `--resolution` supplies a caller-adopted conflict-resolution record; drafting a record alone does not authorize a licence choice. The shipped blocklist has no override. Missing accompanying evidence reported at acquisition is distinct from evidence subsequently lost or altered.

## MCP

The stdio endpoint exposes equivalent operations:

```json
{"name":"fonts_acquire","arguments":{"source":{"kind":"google_fonts","family":"Inter"},"destination":"./acquired"}}
{"name":"fonts_acquire","arguments":{"source":{"kind":"url","url":"https://example.org/font.ttf"},"destination":"./acquired","expected_sha256":"<sha256>"}}
{"name":"fonts_list","arguments":{"roots":["./acquired"],"json":true}}
{"name":"fonts_vendor","arguments":{"project":"project.montagent.json","font":"./acquired/font.ttf","receipt":"./acquired/receipt.json","json":true}}
```

MCP acquisition accepts the same tagged source, destination, expected hash and Face selection as the core contract. Vendor's `destination` corresponds to CLI `--as`; its other optional fields are `licence`, `source`, `resolution`, and `face_index`. These calls run through the blocking-work dispatcher. Font bytes are never returned as base64 in MCP messages.

Paths refer to the machine executing the invoked endpoint. Listing reports endpoint execution identity and the directories actually inspected; it cannot list fonts on another machine. Acquisition results carry execution identity, paths, SHA-256 values, inspected Faces, receipt, findings and next steps. Selection-required results are not successful acquisition; inspect candidates and invoke an explicit selection. Failed acquisition must preserve existing destination content and publish no usable incomplete receipt.

## Evidence and recovery

Preserve acquired files and the receipt until vendoring. The project's optional attestation `evidence: {file, sha256}` points to a manifest containing project-local, hashed references to the original receipt, materials and authorized resolution. Moving the complete project preserves these dependencies; historical receipt paths are provenance, not live dependencies. URLs and hashes do not confer redistribution permission or source authenticity.

Automatic external licence association initially covers selected Google Fonts files and their family materials from one pinned official repository revision. Other sources retain evidence but use recognized embedded metadata or an explicit caller declaration. Unsupported providers require external acquisition of native files and licence materials, followed by explicit local vendor; no adapter or substitute is guessed.

Named remote workspaces and the authenticated HTTP byte door are accepted design, but this checkout ships only local CLI and stdio MCP transport. It does not implement remote acquisition, upload, retrieval or automatic synchronization. A future remote client must explicitly preserve the complete portable bundle in durable agent-controlled storage, verify hashes during transfer, and reconcile project-relative paths. Workspace loss requires explicit upload or reacquisition of exact bytes; changed versions cannot silently substitute.
