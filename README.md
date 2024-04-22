# Penguin

A simple load balancer for HTTP traffic.
Made using Cloudflare Pingora.

Maps any request from "0.0.0.0:6188" to

- `127.0.0.1:8080`
- `127.0.0.1:8888`
- `127.0.0.1:8800`
- `127.0.0.1:8088`
with the round robin algorithm

These endpoints can be configured by using the env vars0

`P_ADDR_ONE`, `P_ADDR_TWO`, `P_ADDR_THREE` and `P_ADDR_FOUR` to the required endpoints.
if no values are provided the defaults are chosen.

> Not all env vars need to be populated, **_Penguin_** has a health check function
> built in. So it will by pass any non active endpoints
