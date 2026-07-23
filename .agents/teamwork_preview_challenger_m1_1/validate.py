import yaml
import sys

with open('/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-forge/SKILL.md', 'r') as f:
    content = f.read()

if not content.startswith('---'):
    print("No YAML frontmatter")
    sys.exit(1)

parts = content.split('---', 2)
if len(parts) < 3:
    print("Invalid YAML frontmatter")
    sys.exit(1)

yaml_part = parts[1]
try:
    parsed = yaml.safe_load(yaml_part)
    print("YAML is valid:", parsed)
except Exception as e:
    print("YAML error:", e)
    sys.exit(1)
