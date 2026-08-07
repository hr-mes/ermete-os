#!/usr/bin/env python3
"""
OpenWiki Spatial Portal - Qdrant Vector Indexer Script
Indexes OpenWiki markdown documentation into Qdrant Vector DB for instant RAG.
"""

import os
import re
import sys
import glob
import json
import math
import requests

QDRANT_HOST = os.getenv("QDRANT_HOST", "http://localhost:6333")
COLLECTION_NAME = os.getenv("QDRANT_COLLECTION", "openwiki_spatial_rag")
EMBEDDING_DIM = 1536  # Standard dimension for OpenAI / OpenRouter text-embedding models

def get_embedding(text: str) -> list[float]:
    """Generates embedding via OpenRouter/OpenAI API or fallback spatial hash."""
    api_key = os.getenv("OPENROUTER_API_KEY") or os.getenv("OPENAI_API_KEY")
    base_url = os.getenv("OPENAI_BASE_URL", "https://openrouter.ai/api/v1")
    
    if api_key:
        try:
            resp = requests.post(
                f"{base_url}/embeddings",
                headers={
                    "Authorization": f"Bearer {api_key}",
                    "Content-Type": "application/json"
                },
                json={
                    "model": "text-embedding-3-small",
                    "input": text[:8000]
                },
                timeout=15
            )
            if resp.status_code == 200:
                data = resp.json()
                return data["data"][0]["embedding"]
        except Exception as e:
            print(f"[W] API embedding failed: {e}, falling back to deterministic spatial vector.", file=sys.stderr)
            
    # Deterministic spatial vector fallback (sine/cosine hash over token frequencies)
    vec = [0.0] * EMBEDDING_DIM
    words = re.findall(r'\w+', text.lower())
    for idx, word in enumerate(words):
        h = sum(ord(c) * (i + 1) for i, c in enumerate(word))
        pos = h % EMBEDDING_DIM
        vec[pos] += math.sin(idx + 1)
    
    # Normalize vector
    norm = math.sqrt(sum(x * x for x in vec)) or 1.0
    return [x / norm for x in vec]

def chunk_markdown(filepath: str) -> list[dict]:
    """Chunks markdown file by headers into spatial RAG payload nodes."""
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()
    except Exception as e:
        print(f"[E] Error reading {filepath}: {e}", file=sys.stderr)
        return []

    chunks = []
    lines = content.splitlines()
    current_header = "Overview"
    current_chunk = []

    for line in lines:
        if line.startswith("#"):
            if current_chunk:
                chunk_text = "\n".join(current_chunk).strip()
                if chunk_text:
                    chunks.append({
                        "file_path": filepath,
                        "header": current_header,
                        "content": chunk_text
                    })
                current_chunk = []
            current_header = line.lstrip("#").strip()
        else:
            current_chunk.append(line)

    if current_chunk:
        chunk_text = "\n".join(current_chunk).strip()
        if chunk_text:
            chunks.append({
                "file_path": filepath,
                "header": current_header,
                "content": chunk_text
            })

    return chunks

def ensure_collection():
    """Ensures Qdrant collection exists."""
    url = f"{QDRANT_HOST}/collections/{COLLECTION_NAME}"
    resp = requests.get(url)
    if resp.status_code != 200:
        print(f"[*] Creating Qdrant collection '{COLLECTION_NAME}'...")
        create_resp = requests.put(
            url,
            json={
                "vectors": {
                    "size": EMBEDDING_DIM,
                    "distance": "Cosine"
                }
            }
        )
        if create_resp.status_code not in (200, 201):
            print(f"[E] Failed to create collection: {create_resp.text}", file=sys.stderr)
            sys.exit(1)
        print(f"[+] Collection '{COLLECTION_NAME}' created successfully.")
    else:
        print(f"[*] Collection '{COLLECTION_NAME}' already exists.")

def index_docs():
    """Finds all markdown files, generates embeddings, and upserts to Qdrant."""
    md_files = []
    for root_dir in ["openwiki", "docs"]:
        if os.path.exists(root_dir):
            md_files.extend(glob.glob(f"{root_dir}/**/*.md", recursive=True))
    
    for extra in ["AGENTS.md", "README.md"]:
        if os.path.exists(extra):
            md_files.append(extra)

    md_files = sorted(list(set(md_files)))
    print(f"[*] Found {len(md_files)} markdown files to index.")

    points = []
    point_id = 1

    for filepath in md_files:
        chunks = chunk_markdown(filepath)
        for chunk in chunks:
            text_to_embed = f"{chunk['file_path']} > {chunk['header']}\n{chunk['content']}"
            vector = get_embedding(text_to_embed)
            points.append({
                "id": point_id,
                "vector": vector,
                "payload": {
                    "file_path": chunk["file_path"],
                    "header": chunk["header"],
                    "content": chunk["content"][:2000], # truncated snippet preview
                    "full_length": len(chunk["content"]),
                    "spatial_layer": "bedrock" if "system" in chunk["file_path"] else "agentic"
                }
            })
            point_id += 1

    if points:
        # Upsert in batches of 100
        batch_size = 100
        total_upserted = 0
        for i in range(0, len(points), batch_size):
            batch = points[i:i + batch_size]
            upsert_resp = requests.put(
                f"{QDRANT_HOST}/collections/{COLLECTION_NAME}/points?wait=true",
                json={"points": batch}
            )
            if upsert_resp.status_code in (200, 201):
                total_upserted += len(batch)
            else:
                print(f"[E] Error upserting batch {i}: {upsert_resp.text}", file=sys.stderr)
        
        print(f"[+] Successfully indexed {total_upserted} vector chunks into Qdrant.")
    else:
        print("[!] No document chunks found to index.")

if __name__ == "__main__":
    print("=== OpenWiki Spatial Portal RAG Indexer ===")
    ensure_collection()
    index_docs()
