#!/usr/bin/env python3
import os, sys, json, re, urllib.request, hashlib, glob
from typing import Dict, Any

WATCH_FILE = "upstream-watch.json"
GITHUB_TOKEN = os.environ.get("GH_TOKEN", "")

def fetch_latest_release(repo: str) -> Dict[str, Any]:
    req = urllib.request.Request(f"https://api.github.com/repos/{repo}/releases/latest")
    if GITHUB_TOKEN: req.add_header("Authorization", f"token {GITHUB_TOKEN}")
    try:
        with urllib.request.urlopen(req) as response:
            return json.loads(response.read().decode())
    except Exception as e:
        # Fallback to tags if no official releases
        req = urllib.request.Request(f"https://api.github.com/repos/{repo}/tags")
        if GITHUB_TOKEN: req.add_header("Authorization", f"token {GITHUB_TOKEN}")
        try:
            with urllib.request.urlopen(req) as response:
                tags = json.loads(response.read().decode())
                if tags: return {"tag_name": tags[0]["name"]}
        except Exception as e2:
            print(f"Failed to fetch upstream for {repo}: {e2}")
    return {}

def download_and_hash(url: str, dest_path: str) -> str:
    print(f"Downloading {url}...")
    urllib.request.urlretrieve(url, dest_path)
    sha256 = hashlib.sha256()
    with open(dest_path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            sha256.update(chunk)
    return sha256.hexdigest()

def update_spec_file(spec_path: str, new_version: str, new_tarball: str):
    if not os.path.exists(spec_path):
        print(f"Warning: {spec_path} does not exist.")
        return False
    with open(spec_path, "r") as f: content = f.read()
    
    current_version = re.search(r'^Version:\s+(.+)$', content, re.M)
    if not current_version: return False
    
    c_ver = current_version.group(1).strip()
    if c_ver == new_version: return False
    
    # Update Version
    content = re.sub(r'^Version:\s+.*$', f'Version:        {new_version}', content, flags=re.M)
    # Reset Release
    content = re.sub(r'^Release:\s+.*$', 'Release:        1%{?dist}', content, flags=re.M)
    
    # Update Source if applicable
    if new_tarball:
        content = re.sub(r'^Source0:\s+.*$', f'Source0:        {new_tarball}', content, flags=re.M)
    
    with open(spec_path, "w") as f: f.write(content)
    return True

def main():
    if not os.path.exists(WATCH_FILE):
        print(f"{WATCH_FILE} not found. Exiting.")
        sys.exit(0)
        
    with open(WATCH_FILE, "r") as f:
        watch_list = json.load(f)
        
    updated = False
    for item in watch_list:
        repo = item.get("repo")
        spec_path = item.get("spec")
        tarball_format = item.get("tarball_format", "{repo_name}-{version}.tar.gz")
        
        print(f"--- Evaluating {repo} ---")
        release = fetch_latest_release(repo)
        if not release: continue
        
        tag = release.get("tag_name", "")
        version = tag.lstrip("v")
        if not version: continue
        
        # Parse current spec to see if update is needed
        try:
            with open(spec_path, "r") as f: current = f.read()
            curr_v = re.search(r'^Version:\s+(.+)$', current, re.M)
            if curr_v and curr_v.group(1).strip() == version:
                print(f"Up to date (v{version}).")
                continue
        except FileNotFoundError:
            continue
            
        print(f"Update found! Upgrading to v{version}")
        
        # Prepare SOURCES directory
        pkg_dir = os.path.dirname(spec_path)
        sources_dir = os.path.join(pkg_dir, "SOURCES")
        os.makedirs(sources_dir, exist_ok=True)
        
        # Clean old sources
        for old_tar in glob.glob(os.path.join(sources_dir, "*.tar.gz")):
            os.remove(old_tar)
            
        # Download new tarball
        repo_name = repo.split("/")[-1]
        tarball_name = tarball_format.format(repo_name=repo_name, version=version, tag=tag)
        download_url = f"https://github.com/{repo}/archive/refs/tags/{tag}.tar.gz"
        dest_path = os.path.join(sources_dir, tarball_name)
        
        sha256sum = download_and_hash(download_url, dest_path)
        print(f"SHA256: {sha256sum}")
        
        if update_spec_file(spec_path, version, tarball_name):
            updated = True
            
    if updated:
        with open(os.environ.get("GITHUB_ENV", "/dev/null"), "a") as env:
            env.write("updates=true\n")
    else:
        with open(os.environ.get("GITHUB_ENV", "/dev/null"), "a") as env:
            env.write("updates=false\n")

if __name__ == "__main__":
    main()
