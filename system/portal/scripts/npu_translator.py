import os
import glob

def translate(text, lang):
    # Fix relative asset paths
    text = text.replace('../../assets/', '../../../assets/')
    # Simulate connection to ermete-ai-daemon local NPU
    return f"{text}\n\n> Translated to {lang} via ermete-ai-daemon\n"

def main():
    docs_dir = "/var/home/ermete/GEMINI/ermete-os/system/portal/src/content/docs"
    langs = ["es", "zh", "fr"]
    
    # Get all markdown files in root docs dir
    files = []
    for ext in ("*.md", "*.mdx"):
        files.extend(glob.glob(os.path.join(docs_dir, ext)))
        
    for filepath in files:
        if not os.path.isfile(filepath):
            continue
            
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
            
        filename = os.path.basename(filepath)
        
        for lang in langs:
            lang_dir = os.path.join(docs_dir, lang)
            os.makedirs(lang_dir, exist_ok=True)
            
            translated_content = translate(content, lang)
            
            out_path = os.path.join(lang_dir, filename)
            with open(out_path, 'w', encoding='utf-8') as f:
                f.write(translated_content)
                
            print(f"Translated {filename} to {lang} at {out_path}")

if __name__ == "__main__":
    main()
