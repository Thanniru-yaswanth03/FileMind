# Contributing to FileMind

Thank you for your interest in contributing to FileMind! FileMind is an open-source, local-first storage intelligence utility and deterministic file safety engine for Windows.

---

## 1. Principles of Contribution

All contributors must review and abide by the [Permanent Engineering Operating Instructions](docs/OPERATING_INSTRUCTIONS.md).

1. **Safety Over Speed:** Never bypass or weaken the `SafetyGuardian` deterministic classification model. "Unknown is Unsafe" is an inviolable architectural invariant.
2. **Local-First & Zero Telemetry:** Never add unconsented remote telemetry, background tracking, or mandatory cloud dependencies.
3. **Impeccable Desktop Craft:** Honor the technical drafting, paper-neutral design system (Warm Ivory / Warm Charcoal). Avoid generic AI visual tropes (neon glows, purple gradients, glassmorphism, cards inside cards).
4. **Reproducible Test Coverage:** Every bug fix or feature addition must include automated regression tests in `crates/filemind-core/tests/`.

---

## 2. Local Development Setup

### Prerequisites
- **Operating System:** Windows 10 (1809+) or Windows 11 (64-bit).
- **Rust Toolchain:** Stable Rust (1.80+) with `x86_64-pc-windows-msvc` or `x86_64-pc-windows-gnu`.
- **Node.js:** Node.js 20+ (LTS) or Node.js 22.
- **Package Manager:** `pnpm` (v9 or v10).

### Initializing the Project
```bash
# Clone the repository
git clone https://github.com/vibe-filemind/filemind.git
cd filemind

# Install frontend dependencies
pnpm install

# Build frontend assets
pnpm run build

# Run core Rust test suite
cargo test --manifest-path crates/filemind-core/Cargo.toml
```

### Running the Development Environment
Run the frontend dev server and native daemon concurrently:

**Terminal 1 (Frontend):**
```bash
pnpm run dev
# Vite runs at http://localhost:3000
```

**Terminal 2 (Native Core Daemon):**
```bash
cargo run --manifest-path crates/filemind-core/Cargo.toml
# Native engine starts at http://127.0.0.1:41732
```

---

## 3. Pull Request Guidelines

1. **Create a Topic Branch:** Create a branch with a descriptive name (`git checkout -b fix/sfn-path-bypass`).
2. **Pre-Flight Validation:** Run the test suite before submitting:
   ```bash
   pnpm run typecheck
   cargo test --manifest-path crates/filemind-core/Cargo.toml --all
   ```
3. **Commit Messages:** Follow standard conventional commit formatting:
   - `fix(safety): patch 8.3 short filename evasion in windows rule engine`
   - `feat(developer): add Gradle and CocoaPods lockfile detection`
   - `perf(scanner): optimize directory entry cycle checking`
4. **No Secrets:** Never commit API keys, personal credentials, or local environment paths.

---

## 4. Code of Conduct

All contributors and maintainers agree to foster an open, welcoming, diverse, and respectful environment. Harassment of any kind is strictly prohibited.

