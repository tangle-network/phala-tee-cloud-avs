# <h1 align="center">Phala Cloud EigenLayer AVS Blueprint ☁️</h1>

## 📚 Overview

This repository contains the off-chain operator blueprint implementation for the Phala Cloud Actively Validated Service (AVS) built on EigenLayer.

The Phala Cloud AVS aims to provide a decentralized cloud computing network where:

- **Providers** (Operators) run TEE-secured (Trusted Execution Environment, e.g., Intel TDX) compute nodes and register with the AVS.
- **Stakers** delegate PHA tokens (or potentially other LSTs via EigenLayer restaking) to operators.
- **Operators** are rewarded for maintaining Service Level Agreements (SLA), primarily liveness and availability, verified through TEE attestations and potentially on-chain challenges.
- **Users** can deploy workloads onto the provider network.
- **Rewards** (from a Phala Reward Pool) and potential **slashing** are managed through the EigenLayer framework and custom AVS smart contracts.

This specific blueprint focuses on the **off-chain operator software** that interacts with the EigenLayer contracts, the Phala AVS contracts, and the TEE environment to fulfill the operator's responsibilities.

It utilizes the [Blueprint SDK](https://github.com/tangle-network/blueprint) to structure the off-chain service, leveraging Eigenlayer-specific components.

## ✨ Key Components

- **Off-Chain Operator Service (Rust):**
  - `phala-tee-cloud-avs-lib`: Core logic including context management, job handlers (SLA checks, challenge responses), TEE interaction, and EVM communication.
  - `phala-tee-cloud-avs-bin`: Binary entrypoint that configures and runs the Blueprint service using `BlueprintRunner`.
  - Uses `alloy-rs` for EVM interactions and `tokio` for asynchronous operations.
- **On-Chain Contracts (Solidity):**
  - `contracts/`: Contains the Solidity smart contracts for the Phala AVS, including:
    - `PhalaServiceManager.sol`: Manages operator registration (via TEE attestations), handles reward proposals from the Tokenomic Manager, and interacts with the SLA Oracle.
    - `PhalaSlaOracle.sol`: Manages SLA challenges and operator responses, reporting failures to the Service Manager.
  - Built using [Foundry](https://getfoundry.sh).
- **EigenLayer Integration:** Leverages EigenLayer's core contracts for staking, delegation, and reward coordination.

## 📋 Prerequisites

Before you can build and run this project, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (including `cargo`)
- [Foundry](https://getfoundry.sh) (including `forge`)
- Docker (Potentially needed for TEE simulation/testing, depending on implementation)

## ⭐ Getting Started

1.  **Clone the repository:**

    ```bash
    git clone <repository-url>
    cd <repository-name>
    ```

2.  **Install Foundry dependencies:**

    ```bash
    forge install
    ```

3.  **Build the contracts:**

    ```bash
    forge build
    ```

4.  **Build the Rust blueprint service:**
    ```bash
    cargo build --release
    ```

## 🛠️ Development & Running

- **Contracts:** Use `forge` commands (`build`, `test`, `script`, `deploy`) to manage the Solidity contracts.
- **Blueprint Service:**
  - Configure necessary environment variables (RPC endpoints, keystore paths, contract addresses, etc.). Refer to `BlueprintEnvironment` usage in `main.rs`.
  - Run the operator service: `cargo run --release --bin phala-tee-cloud-avs-bin`
- **Testing:**
  - Run contract tests: `forge test`
  - Run Rust integration/e2e tests: `cargo test` (Note: E2E tests require Anvil and contract deployments, see `tests/e2e.rs`)

## 📜 License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
