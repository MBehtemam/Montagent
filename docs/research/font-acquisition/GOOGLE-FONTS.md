# Google Fonts acquisition: documented routes and evidence boundaries

Research for [Google Fonts provider research](https://github.com/MBehtemam/Montagent/issues/946), reviewed 11 October 2026. This is a planning artifact, not implementation, a licence determination, or an adapter contract.

## Local constraints

[CONTEXT.md](../../../CONTEXT.md) defines a Font as an ordered chain of files and a Face as the selected outlines and metrics. An Attestation describes a file at a project path. [ADR-0057](../../adr/0057-font-vendoring-licence-gate-and-path-keyed-attestation.md) keeps vendor a local byte copy with a redistribution gate; acquisition must remain separate. [ADR-0102](../../adr/0102-a-ttc-chain-entry-is-cut-down-to-its-face-before-it-is-registered.md) extracts a collection Face only in memory during registration. It does **not** authorize vendor to write extracted files or attest their changed bytes. A variable-axis position is not a collection index; provider selection must not conflate them.

## Documented discovery and download mechanisms

The [Developer API](https://developers.google.com/fonts/docs/developer_api) requires an API key and provides JSON family metadata. Family filtering is documented. Responses describe variants, subsets, family version, last-modified date and variant-to-file URLs covering all supported scripts. Examples use TTF URLs. Default responses supply static instances at standard positions for variable families; `capability=VF` instead returns variable files and axis ranges. `capability=WOFF2` requests compressed web-font files. These are different acquisition choices, not interchangeable names for the same bytes. This documentation provides no requested historical-version parameter or immutable-byte guarantee. The family version and modification date are useful provider assertions, not independently verified content identity.

The official [google/fonts README](https://raw.githubusercontent.com/google/fonts/main/README.md) describes a binary-font repository organized by licence and family. Family directories contain TTFs, metadata, descriptions and appropriate licence files; font binaries also contain authorship/licensing metadata. It documents a whole-repository ZIP snapshot and Git synchronization. The repository is a primary download route independent of the keyed Developer API. The README identifies OFL, Apache 2 and Ubuntu licences, and warns that some OFL fonts have Reserved Font Names. Its redistribution statement remains expressly subject to each licence's terms.

The [CSS2 API](https://developers.google.com/fonts/docs/css2) selects family, individual axis positions or ranges. Defaults include normal/400, with failure when unavailable defaults fall outside a family's supported range. It optimizes requested styles into compact font sets and supports text-specific subsetting. Thus a CSS request is a web-delivery specification, not a promise of one complete editable-project file. [Technical Considerations](https://developers.google.com/fonts/docs/technical_considerations) expressly documents user-agent-specific stylesheets and corresponding font formats. Its browser examples are dated; the useful documented boundary is negotiation, not a current exhaustive format matrix.

No documented automated endpoint for the Google Fonts website's download UI was established here. Avoid treating a private website request as a supported API contract.

## Observed current source, pinned for inspection

The repository revision read through GitHub's API was `bd8f81ddb5c74d5c8897b36ad88b440266245103`. The [Roboto metadata at that revision](https://raw.githubusercontent.com/google/fonts/bd8f81ddb5c74d5c8897b36ad88b440266245103/ofl/roboto/METADATA.pb) lists normal and italic variable TTF files, width/weight axes, copyright, an OFL assertion, and an upstream repository, commit, release archive and file mapping. This is an example of available provenance, not a guarantee all families supply those fields. Its nominal weight 400 does not mean the variable file contains only weight 400.

The [archived metadata specification](https://github.com/googlefonts/gf-docs/tree/main/METADATA) distinguishes family names, file names, style, weight and variable-axis metadata. It explicitly allows font files to contain axes or ranges not advertised in metadata. It is historical reference, not a current compatibility guarantee. Inspect acquired bytes for actual file format, names, Face count and axes rather than trusting filenames or metadata alone.

This research did not download or inspect font binaries, compare repository binaries with service binaries, or establish their byte equality. No new supported-format claim follows: documented repository TTFs fit the intended local route; support for direct-URL OTF/TTC/OTC and rejection or conversion of WOFF/WOFF2 remain Montagent decisions. Neither observed provider route promises Google Fonts collection files.

## Revisions, integrity and source evidence

[GitHub's permanent-link documentation](https://docs.github.com/en/repositories/working-with-files/using-files/getting-permanent-links-to-files) distinguishes mutable branch links from links naming an exact commit. A repository acquisition can resolve a revision once and obtain font, metadata and accompanying materials from that same revision. This identifies a version; it does not promise perpetual remote availability.

Design inference: retain actual bytes and their computed SHA-256, requested/final URLs, retrieval time, source kind, selected repository revision/path or service version assertion, selection request, resolved files and inspected Face/axis facts. Bind retained evidence files to those exact files and record evidence hashes. A hash verifies equality with an expected value; a locally computed hash alone proves neither publisher authenticity nor licence permission. Reusing a mutable URL later is not reproducible acquisition. Offline reuse depends on retained bytes and evidence, not on a saved network locator. Do not record API credentials in the receipt.

## Original materials and licence boundaries

[Roboto's original OFL file at the pinned revision](https://raw.githubusercontent.com/google/fonts/bd8f81ddb5c74d5c8897b36ad88b440266245103/ofl/roboto/OFL.txt) combines copyright with the full OFL text. Condition 2 calls for copyright and licence inclusion with redistributed copies, allowing several presentation forms. Conditions addressing modification and Reserved Font Names mean a font transformation must not be assumed equivalent to copying original bytes. The text distinguishes documents created with the font from Font Software distribution. These are observations of source text, not a conclusion about a particular project or export.

[Apache 2.0 section 4](https://www.apache.org/licenses/LICENSE-2.0.txt) separately addresses providing the licence, changed-file notices, retained notices and NOTICE attribution when applicable. One generic licence identifier cannot preserve all original materials or identify every applicable obligation.

Design inference: collect the original family licence file, relevant copyright/NOTICE/attribution files and source metadata without normalizing away text. Keep provider claims, embedded-name-table evidence and explicit caller declarations separately attributable. A generic direct font URL need not expose a discoverable sibling licence: absence, mismatches and unverifiable association must remain visible. The acquisition receipt reports evidence; vendor owns the established gate and Attestation. The exact evidence schema and conflict policy remain decisions for the licence ticket.

## Decisions now informed, still open

- Choose the initial Google Fonts adapter: keyed Developer API, revision-pinned repository, or an explicitly specified combination. Discovery and byte acquisition need not use the same source, but any cross-source mapping needs evidence rather than a presumed match.
- Specify exact selection: static instance versus variable file; available style/weight choices; axis requests; collection Face selectors; and behavior for ambiguity or unavailable requests. Acquiring a variable file alone does not extend Montagent's rendering semantics to arbitrary axis positions.
- Decide supported payload forms and whether any transformations are permitted. Transformation adds byte-identity, licence-material and Attestation questions beyond acquisition of existing files.
- Define retained evidence, receipt-to-vendor association and handling of missing/conflicting evidence. Existing vendor gates are not repealed by provider membership.
- Define cache identity, expected-hash enforcement, failures and remote transfer. Provider documentation does not decide where Montagent's authoritative project lives.

The cited public documents establish routes and metadata semantics. Service retention, historical version retrieval, licence completeness for every family, universal metadata compatibility and arbitrary direct-URL licence association remain unproven. No private endpoint, credentialed Developer API call or legal adjudication was used.
