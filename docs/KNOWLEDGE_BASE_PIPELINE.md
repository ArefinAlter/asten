# Asten Knowledge-Base Pipeline

This is the product-repo copy of the research retrieval plan. The
implementation lives in `B:\asten-research-ingest`.

See the operational specification:

[`B:\asten-research-ingest\docs\KNOWLEDGE_BASE_PIPELINE.md`](../../asten-research-ingest/docs/KNOWLEDGE_BASE_PIPELINE.md)

## Rule

Do not send the converted library through a paid model. Retrieve 15–40 cited
chunks, synthesize once, and keep the synthesis in an Obsidian vault.

## Stack

- **Qdrant** (local Docker): dense + BM25 hybrid search with RRF
- **FastEmbed** (local): `BAAI/bge-small-en-v1.5` and `Qdrant/bm25`
- **Local reranker:** `Xenova/ms-marco-MiniLM-L-6-v2`
- **Obsidian vault:** `B:\asten-research-ingest\knowledge`

No cloud API keys are required.

## Commands

```powershell
cd B:\asten-research-ingest
docker compose up -d qdrant
.\.venv\Scripts\asten-ingest.exe index corpus
.\.venv\Scripts\asten-ingest.exe search "How should discovery writers interact with canon?" --top 25
.\.venv\Scripts\asten-ingest.exe context-pack "How should discovery writers interact with canon?" --name discovery-writer-canon
```

Then open `B:\asten-research-ingest\knowledge` as an Obsidian vault and write
topic notes from the generated context pack.
