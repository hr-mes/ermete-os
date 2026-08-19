#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
use std::path::{Path, PathBuf};
use std::sync::{RwLock, OnceLock};
use tracing::info;
use serde::{Deserialize, Serialize};

/// Result of static buffer overflow analysis on eBPF bytecode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BufferOverflowValidation {
    pub is_safe: bool,
    pub analyzed_instructions: usize,
    pub max_stack_depth_bytes: u16,
    pub simulated_memory_accesses: usize,
    pub detected_violations: Vec<String>,
}

/// JIT Artifact metadata returned after successful compilation & validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledEbpfArtifact {
    pub patch_id: String,
    pub source_path: String,
    pub output_path: String,
    pub bytecode_size_bytes: usize,
    pub validation: BufferOverflowValidation,
}

/// eBPF Hot-Patch JIT Compiler and Static Verifier
pub struct EbpfJitCompiler {
    output_dir: PathBuf,
    target_triple: String,
}

impl Default for EbpfJitCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl EbpfJitCompiler {
    pub fn new() -> Self {
        Self {
            output_dir: PathBuf::from("/tmp/ermete-patches"),
            target_triple: "bpfel-unknown-none".to_string(),
        }
    }

    pub fn with_output_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: dir.into(),
            target_triple: "bpfel-unknown-none".to_string(),
        }
    }

    fn sanitize_id(patch_id: &str) -> String {
        let sanitized: String = patch_id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();
        if sanitized.is_empty() {
            "patch_default".to_string()
        } else {
            sanitized
        }
    }

    pub fn compile_and_validate(&self, rust_source: &str, raw_patch_id: &str) -> Result<CompiledEbpfArtifact, String> {
        let patch_id = Self::sanitize_id(raw_patch_id);

        if !self.output_dir.exists() {
            std::fs::create_dir_all(&self.output_dir)
                .map_err(|e| format!("Failed to create patch directory {:?}: {}", self.output_dir, e))?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = std::fs::set_permissions(&self.output_dir, std::fs::Permissions::from_mode(0o700)) {
                tracing::error!("Failed to set permissions on output_dir {:?}: {:?}", self.output_dir, e);
            }
        }

        let src_path = self.output_dir.join(format!("{}.rs", patch_id));
        let out_path = self.output_dir.join(format!("{}.o", patch_id));

        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&src_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, rust_source.as_bytes()))
            .map_err(|e| format!("Failed to write source file {:?}: {}", src_path, e))?;

        info!("JIT eBPF Architect: Compiling hot-patch '{}' with rustc --target {}", patch_id, self.target_triple);

        let mut rustc_cmd = std::process::Command::new("rustc");
        rustc_cmd
            .arg("--target").arg(&self.target_triple)
            .arg("--crate-type").arg("cdylib")
            .arg("-O").arg("-C").arg("panic=abort")
            .arg("-o").arg(&out_path)
            .arg(&src_path);

        let output = rustc_cmd.output().map_err(|e| format!("rustc execution failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("rustc compilation error for patch '{}': {}", patch_id, stderr));
        }

        let bytecode = std::fs::read(&out_path)
            .map_err(|e| format!("Failed to read compiled BPF object {:?}: {}", out_path, e))?;

        let validation = self.validate_buffer_overflow(&bytecode)?;
        if !validation.is_safe {
            return Err(format!(
                "Static Buffer Overflow Validation FAILED for patch '{}': {:?}",
                patch_id, validation.detected_violations
            ));
        }

        info!("JIT eBPF Architect: Successfully compiled and validated patch '{}'", patch_id);

        Ok(CompiledEbpfArtifact {
            patch_id,
            source_path: src_path.to_string_lossy().to_string(),
            output_path: out_path.to_string_lossy().to_string(),
            bytecode_size_bytes: bytecode.len(),
            validation,
        })
    }

    pub fn validate_buffer_overflow(&self, bytecode: &[u8]) -> Result<BufferOverflowValidation, String> {
        let instructions = Self::extract_ebpf_instructions(bytecode)?;
        let num_instructions = instructions.len() / 8;
        
        let mut max_stack_depth_bytes = 0;
        let mut simulated_memory_accesses = 0;
        let mut detected_violations = Vec::new();
        
        for i in 0..num_instructions {
            let offset = i * 8;
            let opcode = instructions[offset];
            let class = opcode & 0x07;
            
            if class == 0x01 || class == 0x02 || class == 0x03 {
                simulated_memory_accesses += 1;
                let regs = instructions[offset + 1];
                let dst_reg = regs & 0x0F;
                let src_reg = regs >> 4;
                let off_bytes = [instructions[offset + 2], instructions[offset + 3]];
                let mem_offset = i16::from_le_bytes(off_bytes);
                
                if (class == 0x02 || class == 0x03) && dst_reg == 10 { 
                    if mem_offset >= 0 {
                        detected_violations.push(format!("Stack overflow detected at instruction {}: positive offset {}", i, mem_offset));
                    } else {
                        let abs_offset = (-mem_offset) as u16;
                        if abs_offset > max_stack_depth_bytes { max_stack_depth_bytes = abs_offset; }
                        if mem_offset < -512 {
                            detected_violations.push(format!("Stack overflow detected at instruction {}: offset {} < -512", i, mem_offset));
                        }
                    }
                } else if class == 0x01 && src_reg == 10 { 
                    if mem_offset >= 0 {
                        detected_violations.push(format!("Stack out-of-bounds read at instruction {}: positive offset {}", i, mem_offset));
                    } else {
                        let abs_offset = (-mem_offset) as u16;
                        if abs_offset > max_stack_depth_bytes { max_stack_depth_bytes = abs_offset; }
                        if mem_offset < -512 {
                            detected_violations.push(format!("Stack out-of-bounds read at instruction {}: offset {} < -512", i, mem_offset));
                        }
                    }
                }
            }
        }
        
        let is_safe = detected_violations.is_empty();
        
        Ok(BufferOverflowValidation {
            is_safe,
            analyzed_instructions: num_instructions,
            max_stack_depth_bytes,
            simulated_memory_accesses,
            detected_violations,
        })
    }

    fn extract_ebpf_instructions(bytecode: &[u8]) -> Result<Vec<u8>, String> {
        if bytecode.len() < 8 {
            return Err("Bytecode too small to contain valid eBPF instructions".to_string());
        }
        if bytecode.starts_with(b"\x7fELF") {
            let header_offset = 64;
            if bytecode.len() > header_offset {
                let body = &bytecode[header_offset..];
                let len = (body.len() / 8) * 8;
                return Ok(body[..len].to_vec());
            }
        }
        let len = (bytecode.len() / 8) * 8;
        Ok(bytecode[..len].to_vec())
    }
}

pub struct LivePatchManager {
    compiled_jit_count: RwLock<usize>,
}

static INSTANCE: OnceLock<LivePatchManager> = OnceLock::new();

impl LivePatchManager {
    pub fn global() -> &'static LivePatchManager {
        INSTANCE.get_or_init(|| LivePatchManager {
            compiled_jit_count: RwLock::new(0),
        })
    }

    pub fn jit_compile_patch(&self, rust_source: &str, patch_id: &str) -> Result<CompiledEbpfArtifact, String> {
        let compiler = EbpfJitCompiler::new();
        let artifact = compiler.compile_and_validate(rust_source, patch_id)?;
        if let Ok(mut count) = self.compiled_jit_count.write() {
            *count += 1;
        }
        Ok(artifact)
    }

    /// ZERO-TRUST ENFORCEMENT:
    /// Dynamic shared library loading (`dlopen`) over Zbus is entirely removed to prevent RCE.
    /// All hot patches MUST be JIT-compiled eBPF bytecode and validated by the kernel verifier.
    pub fn load_patch_so(&self, _so_path: &str) -> Result<String, String> {
        Err("SECURITY POLICY VIOLATION: Arbitrary dynamic loading (.so) over D-Bus is strictly forbidden (RCE Vector). Use JIT compiled eBPF patches instead.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_output_dir() {
        let custom_dir = PathBuf::from("/tmp/ermete-patches-test-custom");
        let compiler = EbpfJitCompiler::with_output_dir(&custom_dir);
        assert_eq!(compiler.output_dir, custom_dir);
    }
}
