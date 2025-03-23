# Axiom Documentation

Axiom is a tool for formal verification of AI-generated code, providing a modular, language-agnostic interface that works with various formal verification systems.

**Version:** 0.1.0  
**Author:** Axiom Team  
**Last Updated:** March 22, 2025

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Command Line Interface](#command-line-interface)
  - [Global Options](#global-options)
  - [Commands](#commands)
- [Core Concepts](#core-concepts)
  - [Specifications](#specifications)
  - [Implementation](#implementation)
  - [Verification](#verification)
- [Language Support](#language-support)
- [Verification Systems](#verification-systems)
- [Domain Expertise](#domain-expertise)
- [Advanced Usage](#advanced-usage)
- [Configuration](#configuration)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Overview

Axiom bridges the gap between natural language requirements, formal specifications, and verified implementations. It enables:

- Generation of formal specifications from natural language requirements
- Translation between different verification languages
- Generation of implementation code from formal specifications
- Verification of implementations against their specifications
- Integration with various verification backends (F\*, Dafny, Coq, etc.)

Axiom targets domains where correctness is critical, such as cryptographic algorithms, distributed systems, safety-critical software, and high-assurance applications.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/axiom.git
cd axiom

# Build the project
cargo build --release

# Install globally
cargo install --path .
```

### Using Cargo

```bash
cargo install axiom
```

## Command Line Interface

Axiom provides a comprehensive CLI for interacting with the verification system.

### Global Options

| Option                         | Description                                          |
| ------------------------------ | ---------------------------------------------------- |
| `-l, --log-level <LEVEL>`      | Sets the log level (error, warn, info, debug, trace) |
| `-c, --config <FILE>`          | Path to configuration file                           |
| `-o, --output-format <FORMAT>` | Output format (text, json)                           |

### Commands

#### `axiom init`

Initialize a new Axiom project.

```bash
axiom init [DIRECTORY] --language <LANG> --verification-system <SYSTEM> --domain <DOMAIN>
```

| Option                               | Description                                    |
| ------------------------------------ | ---------------------------------------------- |
| `DIRECTORY`                          | Project directory (default: current directory) |
| `-l, --language <LANG>`              | Target implementation language                 |
| `-v, --verification-system <SYSTEM>` | Verification system to use                     |
| `-d, --domain <DOMAIN>`              | Application domain                             |

#### `axiom spec`

Generate a formal specification from requirements.

```bash
axiom spec --requirements <FILE> --verification-language <LANG> --domain <DOMAIN> [--output <FILE>] [--detail-level <LEVEL>]
```

| Option                               | Description                                                     |
| ------------------------------------ | --------------------------------------------------------------- |
| `-r, --requirements <FILE>`          | Path to requirements file (one requirement per line)            |
| `-v, --verification-language <LANG>` | Verification language to generate (default: fstar)              |
| `-d, --domain <DOMAIN>`              | Domain for the specification                                    |
| `-o, --output <FILE>`                | Output file for the specification                               |
| `-d, --detail-level <LEVEL>`         | Detailed level for specification generation (default: standard) |

#### `axiom validate`

Validate a formal specification.

```bash
axiom validate --spec <FILE> [--depth <DEPTH>] [--requirements <FILE>]
```

| Option                      | Description                                                  |
| --------------------------- | ------------------------------------------------------------ |
| `-s, --spec <FILE>`         | Path to specification file                                   |
| `-d, --depth <DEPTH>`       | Validation depth (basic, typecheck, formal) (default: basic) |
| `-r, --requirements <FILE>` | Requirements file for completeness checking                  |

#### `axiom implement`

Generate implementation from a specification.

```bash
axiom implement --spec <FILE> --language <LANG> [--optimization <LEVEL>] [--output <FILE>] [--comments]
```

| Option                       | Description                                                                   |
| ---------------------------- | ----------------------------------------------------------------------------- |
| `-s, --spec <FILE>`          | Path to specification file                                                    |
| `-l, --language <LANG>`      | Target language for implementation                                            |
| `-o, --optimization <LEVEL>` | Optimization level (none, speed, size, security, readability) (default: none) |
| `-o, --output <FILE>`        | Output file for implementation                                                |
| `--comments`                 | Include implementation comments (default: true)                               |

#### `axiom verify`

Verify an implementation against a specification.

```bash
axiom verify --implementation <FILE> --spec <FILE> [--system <SYSTEM>] [--output <DIR>] [--proof-level <LEVEL>] [--timeout <SECONDS>]
```

| Option                        | Description                                                             |
| ----------------------------- | ----------------------------------------------------------------------- |
| `-i, --implementation <FILE>` | Path to implementation file                                             |
| `-s, --spec <FILE>`           | Path to specification file                                              |
| `-s, --system <SYSTEM>`       | Verification system to use                                              |
| `-o, --output <DIR>`          | Output directory for verification results                               |
| `-p, --proof-level <LEVEL>`   | Proof level (quick, standard, thorough, exhaustive) (default: standard) |
| `-t, --timeout <SECONDS>`     | Timeout in seconds (default: 300)                                       |

#### `axiom process`

Process requirements through the entire pipeline.

```bash
axiom process --requirements <FILE> --language <LANG> --domain <DOMAIN> --output <DIR> [--system <SYSTEM>] [--verification-language <LANG>] [--interactive]
```

| Option                           | Description                              |
| -------------------------------- | ---------------------------------------- |
| `-r, --requirements <FILE>`      | Path to requirements file                |
| `-l, --language <LANG>`          | Target implementation language           |
| `-d, --domain <DOMAIN>`          | Domain for the specification             |
| `-o, --output <DIR>`             | Output directory for all generated files |
| `-s, --system <SYSTEM>`          | Verification system to use               |
| `--verification-language <LANG>` | Verification language to use             |
| `-i, --interactive`              | Interactive mode (default: true)         |

#### `axiom translate`

Translate between verification languages.

```bash
axiom translate --source <FILE> --target-language <LANG> [--output <FILE>]
```

| Option                         | Description                              |
| ------------------------------ | ---------------------------------------- |
| `-s, --source <FILE>`          | Path to source specification file        |
| `-t, --target-language <LANG>` | Target verification language             |
| `-o, --output <FILE>`          | Output file for translated specification |

#### `axiom list`

List supported languages, verification systems, and domains.

```bash
axiom list [WHAT]
```

| Option | Description                                                                                    |
| ------ | ---------------------------------------------------------------------------------------------- |
| `WHAT` | What to list (languages, verification-systems, domains, verification-languages) (default: all) |

#### `axiom check`

Check tool availability and integration.

```bash
axiom check [--system <SYSTEM>] [--language <LANG>] [--install]
```

| Option                  | Description                                               |
| ----------------------- | --------------------------------------------------------- |
| `-s, --system <SYSTEM>` | Verification system to check                              |
| `-l, --language <LANG>` | Implementation language to check                          |
| `-i, --install`         | Install missing dependencies if possible (default: false) |

## Core Concepts

### Specifications

A specification in Axiom formally describes the expected behavior of a program, using logical predicates and formal verification language constructs. Specifications include:

- Pre-conditions: Conditions that must be true before a function executes
- Post-conditions: Conditions that must be true after a function executes
- Invariants: Properties that must always hold
- Type-level guarantees: Properties enforced by the type system

Axiom supports different specification paradigms:

- Pre/post-condition based specifications
- Dependent type theory
- Refinement types
- Contract-based specifications
- State machine specifications

### Implementation

The implementation is the actual code that satisfies the specification. Axiom supports generating and verifying implementations in languages:

- Rust
- C/C++
- Python
- JavaScript
- Go
- Haskell
- OCaml
- Java
- C#
- Scala
- Swift

### Verification

Verification is the process of mathematically proving that an implementation satisfies its specification. Axiom supports various verification approaches:

- Type-level verification
- Automated theorem proving
- Interactive theorem proving
- SMT solver based verification
- Model checking

## Language Support

Axiom supports a wide range of implementation languages, each with different verification capabilities:

| Language | Verification Approaches          | Recommended Systems    |
| -------- | -------------------------------- | ---------------------- |
| Rust     | Refinement types, Contracts      | RustMIRAI, Liquid Rust |
| C/C++    | Pre/post conditions              | Frama-C (ACSL), Why3   |
| Java     | Pre/post conditions              | JML, KeY               |
| Python   | Runtime contracts, Static typing | PyType, Typeguard      |
| Haskell  | Type-level verification          | Liquid Haskell         |
| OCaml    | Type-level verification          | F\*                    |
| F#       | Type-level verification          | F\*                    |

## Verification Systems

Axiom integrates with various verification systems:

| System   | Type                          | Strengths                                  | Ideal For                                   |
| -------- | ----------------------------- | ------------------------------------------ | ------------------------------------------- |
| F\*      | Dependent type system         | Cryptographic verification, Low-level code | Cryptography, Security protocols            |
| Dafny    | Program verifier              | Automated verification, Readable proofs    | General purpose, Teaching                   |
| Coq      | Interactive proof assistant   | Expressive logic, Foundational proofs      | Complex algorithms, Theoretical foundations |
| Isabelle | Interactive proof assistant   | Higher-order logic, Rich libraries         | Complex systems, Mathematical proofs        |
| Lean     | Interactive proof assistant   | Modern interface, Mathlib                  | Mathematical proofs, Formalized mathematics |
| TLA+     | Model checker                 | Distributed systems verification           | Concurrent and distributed systems          |
| Why3     | Program verification platform | Multi-prover support                       | Imperative programs                         |
| Z3       | SMT solver                    | Automated theorem proving                  | Low-level verification conditions           |

## Domain Expertise

Axiom tailors specifications and verification approaches based on the application domain:

| Domain              | Specification Focus                  | Verification Techniques                   |
| ------------------- | ------------------------------------ | ----------------------------------------- |
| Cryptography        | Correctness, Side-channel resistance | Game-based proofs, F\* verification       |
| Distributed Systems | Consensus, Fault tolerance           | Model checking, TLA+                      |
| Web Security        | Authentication, Authorization        | Session models, Information flow          |
| Machine Learning    | Robustness, Fairness                 | Robustness bounds, Invariant verification |
| Systems Software    | Memory safety, Resource management   | Separation logic, Ownership types         |
| Blockchain          | Smart contract correctness           | Transaction invariants                    |
| Safety Control      | Real-time guarantees                 | Time bounds, Response time verification   |
| High Assurance      | Complete specification               | Full functional correctness               |

## Advanced Usage

### Proof Refinement

Proofs often require refinement to address verification gaps:

```bash
# Iteratively refine a proof
axiom verify --implementation impl.rs --spec spec.fst --interactive
```

### Cross-Language Verification

Verify specifications across different languages:

```bash
# Generate a Rust implementation from F* specification
axiom implement --spec crypto.fst --language rust --output crypto.rs

# Verify the implementation
axiom verify --implementation crypto.rs --spec crypto.fst
```

### Pipeline Integration

Integrate Axiom into CI/CD pipelines:

```bash
# Verification as part of CI
axiom verify --implementation src/crypto.rs --spec specs/crypto.fst --output results --output-format json
```

## Configuration

Axiom can be configured using a configuration file:

```toml
# axiom.toml

[system]
log_level = "info"
cache_dir = ".axiom/cache"

[verification]
default_system = "f*"
timeout = 300
proof_level = "standard"

[languages]
default_implementation = "rust"
default_verification = "fstar"

[integrations]
fstar_path = "/usr/local/bin/fstar"
z3_path = "/usr/local/bin/z3"
dafny_path = "/usr/local/bin/dafny"
```

## API Reference

Axiom provides a Rust API for programmatic usage:

```rust
use axiom::{
    AxiomSystem,
    Domain,
    Language,
    VerificationLanguage,
    AxiomOptions,
    VerificationSystem,
    SpecificationOptions,
    ValidationDepth
};

// Create an instance of the Axiom system
let axiom = // Implementation of AxiomSystem

// Define natural language requirements
let requirements = vec![
    "Implement AES-256 encryption".to_string(),
    "The implementation must be resistant to timing attacks".to_string(),
    "The key size must be 256 bits".to_string(),
];

// Set up specification options including verification language
let mut spec_options = SpecificationOptions::default();
spec_options.verification_language = VerificationLanguage::FStarLang;

// Generate formal specification
let formal_spec = axiom.generate_formal_specification(
    &requirements,
    Domain::Cryptography,
    VerificationLanguage::FStarLang,
    &spec_options,
)?;

// Generate implementation from specification
let implementation = axiom.generate_implementation_from_formal_spec(
    &formal_spec,
    Language::Rust,
    &implementation_options,
)?;

// Verify implementation against specification
let verification_result = axiom.verify_against_formal_spec(
    &implementation,
    &formal_spec,
    &verification_options,
)?;
```

## Examples

### Example 1: Verifying a Cryptographic Function

```bash
# Generate a formal specification for AES encryption
axiom spec --requirements crypto_reqs.txt --verification-language fstar --domain cryptography --output aes.fst

# Generate a Rust implementation
axiom implement --spec aes.fst --language rust --output aes.rs

# Verify the implementation
axiom verify --implementation aes.rs --spec aes.fst --system fstar
```

### Example 2: Verifying a Distributed Algorithm

```bash
# Generate a formal specification for Paxos consensus
axiom spec --requirements paxos_reqs.txt --verification-language tla --domain distributed --output paxos.tla

# Validate the specification
axiom validate --spec paxos.tla --depth formal

# Verify against a custom implementation
axiom verify --implementation paxos.go --spec paxos.tla --system tla
```

## Troubleshooting

### Common Issues

#### Verification Timeout

If verification times out, try:

```bash
axiom verify --implementation impl.rs --spec spec.fst --timeout 600 --proof-level quick
```

#### Missing Verification Tools

If verification tools are missing:

```bash
axiom check --system fstar --install
```

#### Specification Generation Failures

For specification generation issues:

```bash
axiom spec --requirements reqs.txt --domain cryptography --verification-language fstar --log-level debug
```

### Getting Help

For more help and examples, visit:

- Documentation: https://axiom-verification.github.io/docs
- Repository: https://github.com/yourusername/axiom
- Issues: https://github.com/yourusername/axiom/issues
