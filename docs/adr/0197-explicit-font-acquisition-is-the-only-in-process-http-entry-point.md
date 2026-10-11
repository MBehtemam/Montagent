---
status: accepted
amends: 0064 (permits an HTTPS client for explicit font acquisition; update checks and telemetry remain forbidden), 0057 (adds retained acquisition evidence to local vendoring)
---

# Explicit font acquisition owns the in-process HTTPS client

The author approved [the font acquisition contract](https://github.com/MBehtemam/Montagent/issues/952#issuecomment-6113673541): CLI and MCP acquire exact native font bytes and accompanying evidence explicitly, then stop. Vendor remains local-only; font validation and rendering never fetch. This makes the former packaging ban on all HTTP-client crates too broad. Permit reqwest and its hyper HTTPS dependencies, and guard their direct use to the acquisition module while retaining the bans on other HTTP clients, telemetry, update checks and linked FFmpeg. Remote media probing remains ffprobe's separate existing authoring-time read.

The rustls trust-anchor dependency webpki-roots carries Mozilla certificate data under [CDLA-Permissive-2.0](https://cdla.dev/permissive-2-0/). Its sharing condition requires retaining the agreement text; generated THIRD-PARTY notices retain it. Add that permissive data licence to the two distribution allowlists without admitting copyleft licences. This records the HTTPS dependency's attribution basis, not permission for any acquired font: those remain governed by the separate font licence/evidence gate.
