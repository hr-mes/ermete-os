# Contributing to Ermete OS

Thank you for your interest in contributing to Ermete OS! We are building a secure, post-quantum, AI-driven operating system. 

## Code of Conduct
This project adheres to the Contributor Covenant. By participating, you are expected to uphold this code. Please report unacceptable behavior to security@ermete-os.org.

## How to Contribute
1. **Discuss Architecture First:** Before opening a PR for a major feature, please open an Issue to discuss the architectural implications. Ermete OS has a strict Zero-Trust and Panics-Free policy.
2. **Kani Formal Verification:** All Ring-0 and IPC code must be formally verified using `kani`. Run the test suite before submitting.
3. **No Unsafe Code:** Do not introduce `unsafe` blocks in Rust unless mathematically proven and strictly isolated.
4. **Sign your commits:** We enforce DCO (Developer Certificate of Origin) and SLSA L4 compliance. Ensure your commits are GPG/SSH signed.

## Development Setup
Check the `docs/` folder for instructions on how to set up the `ermete-os-builder` heavy container for local development.
