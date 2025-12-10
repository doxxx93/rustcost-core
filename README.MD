<h1 align="center" style="border-bottom: none">
    <img alt="rustcost Logo" src="https://avatars.githubusercontent.com/u/233272142" width="120"><br>
    <b>rustcost-core</b>
</h1>

<p align="center">
The backend of <b>RustCost</b> — a lightweight, Kubernetes-native<br>
cost monitoring and observability platform.<br>
Provides API services, data aggregation, and metrics collection.
</p>

<div align="center">

<!-- Badges -->
<img alt="Docker Pulls" src="https://img.shields.io/docker/pulls/rustcost/core.svg">
<img alt="Docker Image Size" src="https://img.shields.io/docker/image-size/rustcost/core/
1.0.0-dev.19">
<img alt="License: Apache-2.0" src="https://img.shields.io/badge/License-Apache--2.0-blue.svg">

</div>

---

## **Overview**

`rustcost-core` is the main backend service powering the RustCost platform.

It is responsible for:

- **Collecting Kubernetes cost data**
- **Aggregating GPU/CPU/memory usage**
- **Serving REST API endpoints** (Axum)
- **Persisting data**
- **Providing metrics** for dashboards & exporters

Built in **Rust**, it is optimized for high throughput,  
low latency, and operational safety.

---

## **Architecture**

RustCost Core consists of:

- **HTTP API Server (Axum)**
- **Background Metrics Collector**
- **File based-backed Storage**

It operates as a single lightweight container and is fully compatible with:

- Kubernetes (Helm)
- Docker standalone deployments

---

## **Deployment Options**

You can deploy `rustcost-core` using **two methods**:

---

# 1. Deploy via Kubernetes (Helm Chart)

This is the **recommended** production deployment method.

Helm chart:  
https://github.com/rustcost/rustcost-helmchart

The chart includes:

- Core backend
- GPU exporter integrations
- Dashboards
- Automatic configuration

---

# **2. Standalone Docker Container**

For testing or small setups:
(TODO - Not Tested yet)
```bash
docker run -d \
  -p 9000:9000 \
  -e DATABASE_URL=postgres://user:pass@host:5432/rustcost \
  rustcost/core:latest
````
---

## **Configuration**

Environment variables:

| Variable       | Required | Description                          |
| -------------- | -------- | ------------------------------------ |
| `DATABASE_URL` | Yes      | PostgreSQL connection string         |
| `RUST_LOG`     | No       | Logging level (`info`, `debug`, etc) |
| `PORT`         | No       | API port (default: `9000`)           |

---

## **Developer Notes**

---

## **Versioning Guideline**

RustCost follows a structured release process:

### **Development Phase**

Frequent builds following:

```
-dev.N
```

Example:

```
0.3.0-dev.4
```

### **Release Candidate Phase**

Used before major production releases:

```
-rc.N
```

Example:

```
1.0.0-rc.2
```

### **Final Production Release**

Stable semantic versioning:

```
1.0.0
```

Planned GA date: **December 21**

---

## **Contributing**

We welcome:

✔ Issues
✔ Feature requests
✔ Pull requests

---

## **License**

Licensed under the **Apache License, Version 2.0**.
