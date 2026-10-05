# Research: the cheapest, quickest way to run a Montagent container on Azure for a first try

Research for [#658](https://github.com/MBehtemam/Montagent/issues/658), part of the map [#656](https://github.com/MBehtemam/Montagent/issues/656).

**Date of research:** 2026-10-05. Montagent at `a57c56e1`; `rmcp` **3.4.0** as pinned (read from `~/.cargo/registry/src/index.crates.io-*/rmcp-3.4.0/`); Azure CLI **2.88.0** (local `-h` output).

**Sourcing rule applied:** every claim cites learn.microsoft.com, the Azure Retail Prices API (`prices.azure.com`, the first-party feed behind the pricing pages, which render their numbers client-side and could not be read directly), the Azure CLI's own help, or source code at a pinned version. Each finding is tagged **[verified]** (read in a primary source) or **[inference]** (reasoned from verified facts, not observed). Prices are **East US, pay-as-you-go, USD**, queried 2026-10-05; other regions differ.

---

## 1. Answer

**Use Azure Container Apps (Consumption plan), image pulled from a public GHCR package, ephemeral disk, `--min-replicas 0 --max-replicas 1`, 4 vCPU / 8 GiB.**

- It is the only one of the three that has **both** a public HTTPS URL out of the box **and** scale-to-zero.
- Its monthly free grant covers about **12.5 hours of a 4-vCPU replica** (25 h at 2 vCPU). Above that, a 4-vCPU replica costs **about $0.43 per hour**, and nothing at all while it is scaled to zero. A month of evening experiments costs single-digit dollars. The $200 credit is not the constraint.
- It takes **three `az` commands** after a one-time setup, and `az group delete` removes everything (§6).

**The one real risk is the 240-second ingress timeout.** Everything Microsoft publishes says it is an *idle* timeout, so a stream that keeps sending bytes should survive a multi-minute render. Two things are already in place to send those bytes: Montagent's 30-second progress heartbeat, and `rmcp`'s 15-second SSE keep-alive. Neither has been tested through Azure's ingress. **The first experiment should run a render longer than 5 minutes over SSE and see whether it survives** (§3).

The runners-up:

- **Container Instances** is cheaper per running hour ($0.20/h at 4 vCPU / 8 GB). But it has no built-in HTTPS (you add a Caddy sidecar plus a file share for its state) and it never scales to zero. Left on by mistake, it costs **about $144 a month**.
- **App Service** is a flat **$49 a month** (B3, 4 cores) whether or not you use it. It has a documented **non-configurable** 240 s timeout on Linux and no scale-to-zero.

---

## 2. Comparison

| | **Container Apps** (Consumption) | **Container Instances** | **App Service for Containers** (Linux) |
|---|---|---|---|
| Public HTTPS, no auth | **Yes, built in.** FQDN, TLS 1.2/1.3 terminated at ingress, port 80 redirects to 443 [verified ¹] | **No TLS built in.** Public IP + `dnsNameLabel` (`<label>.<region>.azurecontainer.io`). Microsoft's own HTTPS recipe is a Caddy sidecar + Let's Encrypt + three Azure Files shares [verified ⁸] | Yes (`*.azurewebsites.net`) |
| HTTP/2, streaming | HTTP/1.1, HTTP/2, WebSocket, gRPC. SSE not named [verified ¹]. SSE is a plain long HTTP response, so it should pass [inference] | Traffic goes straight to the container. No proxy timeout is documented [inference: none found] | Not researched in depth (ruled out on timeout and cost) |
| Request / idle timeout | **240 s** "request time out" [verified ¹]. Only **premium ingress** makes it configurable, as `requestIdleTimeout` (4–30 min). That needs a dedicated D4+ profile with **≥ 2 nodes** [verified ²] | None documented | **230 s (Windows) / 240 s (Linux)**: "cannot be configured" per Microsoft's guidance. Microsoft's fix is WebJobs [verified ⁹] |
| Scale to zero | **Default** (min 0, max 10). Scales in 300 s after the last request. No charge at zero [verified ³ ⁴] | No. Billed from image pull until the group is stopped. `az container stop` stops the meter [verified ⁷] | No. The plan bills per hour while it exists |
| Max size | 4 vCPU / 8 GiB per app on the Consumption profile in a workload-profiles env (now the default env type). A legacy *Consumption-only* env caps at 2 vCPU / 4 GiB [verified ⁵ ¹⁰] | 4 vCPU / 16 GB without trouble. Up to 31 vCPU / 240 GB, subject to regional capacity [verified ¹¹] | B3: 4 cores / 7 GB [verified ¹²] |
| Disk | Ephemeral **8 GiB** per replica above 1 vCPU. Lost when the replica stops. Azure Files (SMB) mount supported [verified ⁶] | Container FS (50 GB max). Azure Files volume mount supported [verified ¹¹ ⁸] | 10 GB (B tier) [verified ¹²] |
| Image source | Any public or private registry. Linux amd64 only [verified ⁵] | ACR or any public OCI registry. amd64 only [verified ⁷] | ACR / public registries |
| Price, 4 vCPU + 8 GiB | **$0.000024/vCPU-s + $0.000003/GiB-s ≈ $0.432/h** active. Free each month per subscription: 180,000 vCPU-s, 360,000 GiB-s, 2 M requests [verified ⁴ ¹³] | **$0.0405/vCPU-h + $0.00445/GB-h ≈ $0.198/h**. vCPU rounded **up** to a whole number per group [verified ¹³ ¹⁴] | **B3 $0.067/h ≈ $49/month** flat (B2: 2 cores, $0.034/h) [verified ¹³ ¹²] |
| Fewest CLI steps | 3 after setup (§6) | 1 `az container create --file` + YAML + storage account/shares for Caddy (≈ 6) | plan + webapp + config (≈ 3), but ruled out |

### Cost at light use, 4 vCPU / 8 GiB [inference: arithmetic on the rates above]

| Usage per month | Container Apps | Container Instances | App Service B3 |
|---|---|---|---|
| 10 h | **$0** (inside free grant) | $2 | $49 |
| 40 h | **$11.9** ((40 − 12.5) × $0.432) | $7.9 | $49 |
| left on 24/7 (730 h) | n/a: scales to zero on its own | **$144** | $49 |

ACI only wins on price if you are disciplined about `az container stop`, and it still lacks HTTPS. At 2 vCPU / 4 GiB, ACA's free grant covers 25 h and the rate halves to $0.216/h.

Small extras, all well under $1 a month at experiment scale: Azure Files Standard LRS **$0.06/GB-month** plus per-10K operations [verified ¹³]. **ACR Basic costs $0.1666/day ≈ $5/month** [verified ¹³], which is why GHCR is preferred (§5). Log Analytics is avoided entirely with `--logs-destination none` [verified: `az containerapp env create -h`].

---

## 3. Will a multi-minute render survive ACA's 240 s timeout?

What is verified:

- The default ingress has a 240 s "request time out" [verified ¹]. Premium ingress, the only way to change it, names that same knob `requestIdleTimeout`: "the time … a request can remain idle before being disconnected", default **4** min, range 4–30 [verified ²]. App Service's 240 s comes from the Azure Load Balancer's **idle** timeout too [verified ⁹].
- Montagent already sends an MCP progress **heartbeat every 30 s** from dispatch to result. It only does so when the client supplied a `progressToken`: `crates/montagent/src/mcp/dispatch.rs:50` (`HEARTBEAT`), `:314` (no token, no notification) [verified].
- `rmcp` 3.4.0's Streamable HTTP server sends an **SSE keep-alive ping every 15 s** by default (`StreamableHttpServerConfig::default`, `tower.rs:190`, `sse_keep_alive: Some(15s)`). That holds only while `json_response` stays `false` (the default) [verified].

[inference] Either source of bytes is well inside 240 s, so an SSE response stream should never look idle to ACA's ingress, even with no `progressToken`. **Not verified:**

- whether ACA's 240 s is really idle-based on the *default* ingress (the docs call it "request time out" there);
- whether ACA's Envoy ingress buffers SSE.

GitHub issues [microsoft/azure-container-apps#1637](https://github.com/microsoft/azure-container-apps/issues/1637) and [#1543](https://github.com/microsoft/azure-container-apps/issues/1543) ask about this timeout and have no staff answer in the visible thread. The prototype's first act should be `curl -N` against an SSE `render` longer than 5 minutes.

If that fails, the fallbacks in cost order are:

1. async job + poll (the map's "long renders over HTTP" question);
2. ACI + Caddy (no ingress proxy);
3. premium ingress. That needs 2 × D4 nodes always on, which is far outside "cheap": the dedicated-plan meter is $0.057/vCPU-h + $0.005/GiB-h plus $0.10/h management [verified ¹³].

---

## 4. Storage

- **First experiment: ephemeral.** 8 GiB per replica at > 1 vCPU [verified ⁶] holds projects, media, the probe cache and `out/`. Everything is lost when the app scales to zero (300 s after the last request [verified ³]). That is acceptable for a no-auth throwaway, and it reveals the persistence question honestly.
- **If a session must survive scale-to-zero:** mount an Azure Files **SMB** share. NFS needs a custom VNet [verified ⁶]. From the CLI this costs a storage account, a share, `az containerapp env storage set …`, and a YAML export/edit/update of the app [verified ⁶]. Expect SMB to be much slower than local disk for ffmpeg intermediates [inference]. Keep scratch on ephemeral disk and put only inputs and outputs on the share.

---

## 5. Getting the image there

- ACA pulls from "any public or private container registry" [verified ⁵]. A **public GHCR package needs no credentials and no Azure resource**. ACR Basic costs about $5/month [verified ¹³].
- The image must be **linux/amd64** [verified ⁵ ⁷]. The dev machine is Apple Silicon (arm64), and cross-building a Rust binary under QEMU is slow [inference]. Two cheap native-amd64 builders:
  - **GitHub Actions** on `ubuntu-latest` pushing to `ghcr.io/mbehtemam/montagent:<sha>`. The repo is public, so the minutes are free.
  - `az acr build`, which builds in Azure but needs an ACR.

  GHCR via Actions also fits the map's "official image (GHCR?)" note.
- Avoid Docker Hub: ACA warns its pull rate limit makes containers fail to start [verified ⁵].

---

## 6. The fewest steps (ACA)

One-time setup:

```sh
az extension add --name containerapp --upgrade
az provider register --namespace Microsoft.App
```

Deploy:

```sh
az group create -n montagent-exp -l eastus
az containerapp env create -n montagent-env -g montagent-exp -l eastus --logs-destination none
az containerapp create -n montagent -g montagent-exp --environment montagent-env \
  --image ghcr.io/mbehtemam/montagent:<tag> --workload-profile-name Consumption \
  --cpu 4 --memory 8Gi --min-replicas 0 --max-replicas 1 \
  --ingress external --target-port 8080 --query properties.configuration.ingress.fqdn
```

Tear down: `az group delete -n montagent-exp`.

Notes:

- `az containerapp env create` defaults to `--enable-workload-profiles true` [verified: CLI 2.88 help], which is what allows 4 vCPU on Consumption [verified ⁵].
- The CLI's own `--cpu` help still says "0.25 – 2.0" [verified: CLI help]. That is stale relative to the docs' 4.0 / 8.0 Gi table. If the CLI rejects `--cpu 4`, use 2 / 4Gi for the first run.
- `az containerapp up` is shorter, but it exposes no `--cpu/--memory` [verified: CLI help], so it would deploy at the default size.
- `--max-replicas 1` matters: Streamable HTTP sessions live in process memory, so a second replica would not know the first one's sessions [inference].

---

## 7. Montagent-side prerequisites this surfaced

These belong to the prototype ticket, not to Azure:

1. **No HTTP transport exists yet.** `crates/montagent/Cargo.toml:20` enables `rmcp` features `server, macros, schemars, transport-io`, which is stdio only [verified].
2. **`rmcp` rejects non-loopback `Host` headers by default.** `allowed_hosts` defaults to `localhost, 127.0.0.1, ::1`, and "public deployments should override this list" (`tower.rs:101–110, 195`) [verified]. Behind ACA the `Host` is the app's FQDN, so the server must allow it, or `disable_allowed_hosts()` for the throwaway.
3. **Billing while a client is connected.** [inference] An MCP client holding open a standalone GET SSE stream is an in-flight HTTP request. The replica will not scale to zero, and it bills at the **active** rate (the cheaper idle rate applies only when `minReplicas > 0` [verified ⁴]), until the client disconnects plus 300 s. Close the client when done.
4. Bind to `0.0.0.0:<target-port>` [inference: standard for ACA ingress].

---

## Sources

1. [Ingress in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/ingress-overview), § Protocol types → HTTP
2. [Use premium ingress in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/premium-ingress); [Configure ingress in an environment](https://learn.microsoft.com/en-us/azure/container-apps/ingress-environment-configuration)
3. [Scaling in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/scale-app), § Default scale rule, § Scale behavior
4. [Billing in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/billing)
5. [Containers in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/containers), § vCPU and memory allocation requirements, § Container registries
6. [Use storage mounts in Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/storage-mounts)
7. [Azure Container Instances FAQ](https://learn.microsoft.com/en-us/azure/container-instances/container-instances-faq), § Pricing, § registries, § ARM64
8. [Enable automatic HTTPS with Caddy as a sidecar (ACI)](https://learn.microsoft.com/en-us/azure/container-instances/container-instances-container-group-automatic-ssl)
9. [Web request times out in Azure App Service](https://learn.microsoft.com/en-us/troubleshoot/azure/app-service/web-request-times-out-app-service). "cannot be configured" is from Microsoft's Q&A answer [1167766](https://learn.microsoft.com/en-us/answers/questions/1167766/azure-app-service-timing-out-in-230-seconds); the troubleshooting page itself only offers WebJobs.
10. [Quickstart: deploy an existing container image](https://learn.microsoft.com/en-us/azure/container-apps/get-started-existing-container-image)
11. [ACI resource availability & quota limits](https://learn.microsoft.com/en-us/azure/container-instances/container-instances-resource-and-quota-limits)
12. [App Service Linux pricing](https://azure.microsoft.com/en-us/pricing/details/app-service/linux/) (instance specs)
13. Azure Retail Prices API, `https://prices.azure.com/api/retail/prices`, filtered `armRegionName eq 'eastus'` for `Azure Container Apps`, `Container Instances`, `Azure App Service`, `Container Registry`, `Storage` (`Files v2`), queried 2026-10-05
14. [Container Instances pricing](https://azure.microsoft.com/en-us/pricing/details/container-instances/) (billing rule text; the rates did not render)
