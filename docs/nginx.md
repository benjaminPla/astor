# nginx configuration

astor is built to run behind a reverse proxy. The settings below are
**required** — astor assumes this work has already been done and does not
re-implement it.

---

## Required

### `proxy_buffering on`

astor reads `Content-Length`-framed bodies only. This is the nginx default,
but it must not be disabled.

```nginx
proxy_buffering on;
```

Without it, nginx forwards chunked bodies with no `Content-Length` header.
astor cannot parse these and will drop the body silently.

### `proxy_http_version 1.1` + `proxy_set_header Connection ""`

astor loops on each TCP connection until nginx closes it (keep-alive). Without
these two lines nginx defaults to HTTP/1.0 and sends `Connection: close`,
which closes the connection after every single request — no pooling, pure TCP
churn.

```nginx
proxy_http_version 1.1;
proxy_set_header   Connection "";
```

### Body size limit

nginx enforces the body size limit before the request reaches astor. astor
does not check it. Without this, a client can stream an arbitrarily large body.

```nginx
client_max_body_size 10m;  # adjust to your workload
```

### Header size limit

nginx enforces header size limits before the request reaches astor. The
defaults (`client_header_buffer_size 1k`, `large_client_header_buffers 4 8k`)
are usually fine. Raise them only if you forward large cookies or tokens.

```nginx
client_header_buffer_size    1k;
large_client_header_buffers  4 8k;
```

### Slow-client timeouts

nginx drops slow clients before they reach astor. astor has no timeout logic —
it trusts the proxy to handle this.

```nginx
client_body_timeout   30s;
client_header_timeout 10s;
```

### Method whitelist

nginx forwards any method string to upstream by default — `ANYTHING /path
HTTP/1.1` gets proxied unchanged. Filter to the methods your service actually
handles and return 405 for everything else.

```nginx
# Example — adjust this list to the methods your service handles.
# The regex is case-sensitive (~, not ~*). HTTP methods must be uppercase
# per RFC 9110 §9.1. astor does not normalise case — it assumes nginx already
# enforces the standard. A client sending `get` instead of `GET` is violating
# the spec and will get a 405 here, before reaching astor.
if ($request_method !~ ^(GET|HEAD|POST|PUT|PATCH|DELETE|OPTIONS)$) {
    return 405;
}
```

---

## Header names

nginx lowercases all forwarded header names before passing them to astor
(e.g. `Authorization` → `authorization`). astor's `req.header()` does
case-insensitive matching, so both forms work — but on the wire from nginx
they will always be lowercase.

---

## Minimal upstream block

```nginx
upstream astor {
    server 127.0.0.1:3000;

    keepalive          64;    # idle connections per worker
    keepalive_requests 1000;  # recycle after N requests
    keepalive_timeout  60s;   # close idle connections after this long
}

server {
    listen 80;
    server_name example.com;

    # Required — adjust to your workload
    client_max_body_size     10m;
    client_body_timeout      30s;
    client_header_timeout    10s;
    client_header_buffer_size    1k;
    large_client_header_buffers  4 8k;

    # Required — example method list, adjust to your service
    if ($request_method !~ ^(GET|HEAD|POST|PUT|PATCH|DELETE|OPTIONS)$) {
        return 405;
    }

    location / {
        proxy_pass         http://astor;

        # Required for keep-alive
        proxy_http_version 1.1;
        proxy_set_header   Connection "";

        # Required — do not disable
        proxy_buffering    on;

        # Forward real client info
        proxy_set_header   Host            $host;
        proxy_set_header   X-Real-IP       $remote_addr;
        proxy_set_header   X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

---

## Kubernetes (ingress-nginx)

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: astor
  annotations:
    nginx.ingress.kubernetes.io/proxy-buffering: "on"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/server-snippet: |
      client_body_timeout   30s;
      client_header_timeout 10s;

      # Example method list — adjust to your service. Case-sensitive — see note above.
      if ($request_method !~ ^(GET|HEAD|POST|PUT|PATCH|DELETE|OPTIONS)$) {
        return 405;
      }
spec:
  rules:
    - host: example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: astor
                port:
                  number: 3000
```

**Required:** set `terminationGracePeriodSeconds` longer than your slowest
request. On `SIGTERM`, astor stops accepting new connections and drains
in-flight requests. If k8s sends `SIGKILL` before the drain finishes, those
requests are dropped — that is not graceful shutdown.

```yaml
spec:
  terminationGracePeriodSeconds: 30  # must be longer than your slowest request
  containers:
    - name: app
      image: your-registry/your-app:latest
      livenessProbe:
        httpGet: { path: /healthz, port: 3000 }
      readinessProbe:
        httpGet: { path: /readyz, port: 3000 }
```
