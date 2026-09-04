# Nested guide

Relative CSS and the Markdown source link should still work from this folder.

Back to [home](../index.html).

```mermaid
sequenceDiagram
  participant U as You
  participant C as cargo
  participant M as mdsite
  participant P as GitHub Pages
  U->>C: test + build
  C->>M: mdsite build
  M->>P: deploy _site
```

Here's the original example used by Simon Willison https://tools.simonwillison.net/grok-mermaid.

Good to compare to see if it looks the same

```mermaid
graph TD
  Start[Request received] --> Auth{Authenticated?}
  Auth -->|yes| Rate{Rate limit OK?}
  Auth -->|no| R401[401 Unauthorized]
  Rate -->|yes| H(Handle request)
  Rate -->|no| R429[429 Too Many Requests]
  H -.-> Log[Audit log]
  H ==> Resp[200 OK]
```


Sub Graphs
```mermaid
flowchart LR
  subgraph Client
    UI[Browser UI] --> SW[Service worker]
  end
  subgraph Server
    API[API gateway] --> DB[Postgres]
  end
  SW -->|HTTPS| API
```


Unsupported
```mermaid
pie title Pets
  "Dogs" : 386
  "Cats" : 85
  ```
