# ==============================================================================
# 🌋 Ermete OS - Main Workspace Task Runner (Justfile)
# Centralized entrypoint for Forge build system and System image builder
# ==============================================================================

mod forge 'forge/Justfile'
mod system 'system/Justfile'

[private]
default:
    @just --list
