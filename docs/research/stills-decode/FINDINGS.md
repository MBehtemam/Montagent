# Do stills decode once? (#648)

Spike for [#648](https://github.com/MBehtemam/Montagent/issues/648), part of the map [#641](https://github.com/MBehtemam/Montagent/issues/641). Measured 2026-10-04 on the M1 Pro, at load 8–14 (observed runs, not the verdict protocol).

`crates/montagent-render/src/canvas.rs` on this branch carries a `MONTAGENT_SPIKE_STILLS` switch. It is never meant for `main`.

- **unset:** main's behaviour, a lazy `Image::from_encoded`;
- **`raster`:** `to_raster_image(CachingHint::Disallow)`;
- **`cache`:** `graphics::set_resource_cache_total_bytes_limit(1 GiB)`.

One binary, built from `14380e0b`, on `fixtures/benchmark/spy-trailer/` (1,080 frames):

| variant | frame hashes | decode samples (`sample`, 1 ms) | total samples | wall, 1 observed run | peak RSS |
|---|---|---|---|---|---|
| main | identical | 26,243 (10.7%) | 244.7k | 261.9 s | 492 MB |
| raster | identical | 63 | 216k (−12%) | 241.2 s | 492 MB |
| cache | identical | 65 | 215k (−12%) | 232.3 s | 492 MB |

- **Skia's default resource-cache limit is 32 MiB** (33,554,432 bytes, logged at run time). The trailer's decoded stills total about 100 MB.
- **skia-safe declares `Image` `Send + Sync`** (`unsafe_send_sync!(Image)`), so K painters can share one decoded still.
- The wall-time gap between raster and cache is within noise. The two are equal on profile totals.

**Decision: raster.** It won a unanimous court, and the dev adopted it. See the resolution on #648.
