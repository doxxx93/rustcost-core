

Developer notes, build instructions, scripts


```bash
cd C:\code\rustcost\rustcost-core
docker build --build-arg APP_NAME=rustcost-core -t rustcost-core .
docker tag rustcost-core rustcost/core:1.0.0-dev.19
docker push rustcost/core:1.0.0-dev.19
```