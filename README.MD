<!-- Force Inter for README content (GitHub may ignore external font loads; inline styles are applied) -->
<div style="font-family: 'Inter', system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;">

<p align="center">
  <img alt="rustcost Logo" src="https://avatars.githubusercontent.com/u/233272142" width="400" /><br />
  <strong id="rc-title" style="font-size:40px;">RUSTCOST-CORE</strong>
</p>

<div align="center">

<!-- Badges -->
<img alt="Docker Pulls" src="https://img.shields.io/docker/pulls/rustcost/core.svg">
<img alt="Docker Image Size" src="https://img.shields.io/docker/image-size/rustcost/core/1.0.0-dev.19">
<img alt="License: Apache-2.0" src="https://img.shields.io/badge/License-Apache--2.0-blue.svg">

</div>

---

![Docker Pulls](https://img.shields.io/docker/pulls/rustcost/core.svg) ![Docker Image Size](https://img.shields.io/docker/image-size/rustcost/core/1.0.0-dev.16) ![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)

RustCost Core is a lightweight, Kubernetes-native backend that ingests cluster resource and billing data, normalizes and aggregates CPU/GPU/memory utilization and cost metrics, and exposes RESTful APIs plus Prometheus-compatible metrics for dashboards and exporters. It supports pluggable persistence (file or SQL adapters), scheduler-driven collectors, Helm deployment, and is optimized for high-throughput telemetry and cost-allocation workflows.

---

## Quick Start

Requirements

- Rust toolchain (`rustup`, stable toolchain recommended)

Run locally (build & run):

```bash
# build
cargo build --release

# run (example; port shown matches code defaults)
HOST=127.0.0.1 PORT=3000 RUSTCOST_LOG_LEVEL=info cargo run --release
```

Run with Docker (example):

```bash
docker run -d \
  -p 3000:3000 \
  -e PORT=3000 \
  -e RUSTCOST_LOG_LEVEL=info \
  rustcost/core:1.0.0-dev.16
```

Run with Helm (example):

```bash
helm repo add rustcost https://rustcost.github.io/helm-charts
helm install rustcost-core rustcost/rustcost-core
```

For production-grade deployment we recommend the Helm chart: https://github.com/rustcost/rustcost-helmchart

---

## Configuration (Environment Variables)

These environment variables are read by the application (defaults shown):

| Variable               | Required | Default       | Description |
|-----------------------:|:--------:|:-------------:|:------------|
| `HOST`                 | No       | `127.0.0.1`   | Server host/address (see `src/config.rs`) |
| `PORT`                 | No       | `3000`        | API port (see `src/config.rs`) |
| `RUSTCOST_DEBUG_MODE`  | No       | `false`       | If `1` or `true`, runs in debug mode (see `src/main.rs`) |
| `RUSTCOST_LOG_DIR`     | No       | `<data>/logs` | Log directory used by tracing (see `src/logging.rs`) |
| `RUSTCOST_LOG_LEVEL`   | No       | `info`        | Log level for tracing (see `src/logging.rs`) |
| `DATABASE_URL`         | No       | —             | Optional Postgres DSN — only required if you enable Postgres-backed adapters |

Notes:

- The project defaults to a file-based persistence adapter; `DATABASE_URL` is optional and
  only required when using Postgres-backed persistence implementations.

---

## Architecture

```
[Kubernetes Cluster] --> [RustCost Core]
                              |
                    +---------+---------+
                    |                   |
          [HTTP API Server]   [Background Scheduler]
              (Axum)              (Collectors)
                    |                   |
            RESTful APIs       Metrics Aggregation
         Prometheus Metrics    Cost Calculations
                    |                   |
            +-------+-------+   +-------+-------+
            | Pluggable      |   | File/SQL       |
            | Persistence    |   | Adapters       |
            +----------------+   +---------------+
```

- HTTP API Server (Axum)
- Background scheduler and metrics collectors
- Pluggable persistence (file-based adapters by default; DB adapters available)

The service is designed to run as a single lightweight container and integrates with Kubernetes via Helm.

---

## Usage / API

At minimum the service exposes a health endpoint and REST API routes for metrics/info.
Please refer to the `src/api/routes` implementations for full route list and request/response models.

Example health check:

```bash
curl http://127.0.0.1:3000/health
```

(Add OpenAPI/Swagger link here if you publish a spec.)

---

## Deployment

Helm

- See https://github.com/rustcost/rustcost-helmchart for chart, recommended production values and examples.

Docker

- Images are published under `rustcost/core` — update tags per your CI workflow.

---

## Development

- Formatting: `cargo fmt`
- Linting: `cargo clippy`
- Tests: `cargo test`

For local development involving other services, consider using `docker-compose` to bring up dependent services (Postgres, exporters) if you enable DB-backed adapters.

---

## **Contributing**

We welcome:

✔ Issues
✔ Feature requests
✔ Pull requests

---

## License

Licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.
