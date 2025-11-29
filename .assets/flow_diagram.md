---
config:
  flowchart:
    nodeSpacing: 15
    rankSpacing: 50
    curve: monotoneX
  layout: fixed
---
flowchart LR
 subgraph S1["1. Input Source"]
        A[("📂 Codebase & Git")]
  end
 subgraph S2["2. Code2Prompt Core"]
    direction LR
        B{"🔍 Filtrage & Config"}
        C["🧠 Smart Processing
(Parse CSV, Notebooks, JSONL)"]
        D["🎨 Templating Layer
(Handlebars + Token Count)"]
  end
 subgraph S3["3. Delivery Interfaces"]
    direction TB
        E["💻 CLI / TUI"]
        F["🐍 Python SDK"]
        G["🔌 MCP Server"]
  end
    A --> B
    B --> C
    C --> D
    D --> E & F & G
    E --> H("🤖 LLM / AI Model")
    F --> H
    G --> H
    H -. 📝 Generate &amp; <br> Integrate Code .-> A

     A:::input
     B:::core
     C:::core
     D:::core
     E:::delivery
     F:::delivery
     G:::delivery
     H:::ai
    classDef input fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef core fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px
    classDef delivery fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef ai fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef loop fill:#ffffff,stroke:#333,stroke-width:1px,stroke-dasharray: 5 5