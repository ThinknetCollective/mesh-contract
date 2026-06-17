# 🔗 Mesh Contract

**Smart contracts powering the ThinkMesh protocol — on-chain reputation, rewards, and governance.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange)](https://www.rust-lang.org/)
[![Soroban](https://img.shields.io/badge/Soroban-20.0.0-purple)](https://soroban.stellar.org/)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Soroban](https://img.shields.io/badge/Soroban-20.0.0-blue)](https://soroban.stellar.org/)
[![Stellar](https://img.shields.io/badge/Stellar-Testnet-7C3AED)](https://stellar.org/)
[![Drips Wave](https://img.shields.io/badge/💧%20Drips-Wave%201%20Active-7C3AED)](https://www.drips.network/)

---

> **🌊 Drips Wave 1 is LIVE — Support mesh-contract and earn your place on the leaderboard!**
> [**Fund this project on Drips →**](https://www.drips.network/app/projects/github/ThinknetCollective/mesh-contract)

---

## 🎯 What is Mesh Contract?

Mesh Contract is the smart contract layer of the [ThinkMesh](https://github.com/ThinknetCollective/thinkmesh-api) ecosystem. It provides:

- **🏆 On-chain reputation** — Immutable proof of contribution quality
- **💰 THINK token** — ERC-20 reward token for problem solvers
- **🎖️ Impact NFTs** — Soulbound badges proving real-world impact
- **🏛️ DAO governance** — Community control over platform decisions
- **💸 Solution bounties** — Smart contract escrow for funded challenges
- **📊 Drips integration** — Streaming rewards to contributors

Think: **"The on-chain trust layer for real-world problem solving"**

---

## 💧 Drips Wave — Fund the Future of Collaborative Problem Solving

> **Wave 1 is open. Your support makes collaborative problem solving unstoppable.**

### Why Fund Mesh Contract?

ThinkMesh is building the infrastructure to reward people who solve real-world problems — from local infrastructure to global challenges. The smart contract layer is the backbone of that vision.

Your contribution via [Drips](https://www.drips.network/) streams directly to the developers building:

| What You Fund | Impact |
|---|---|
| 🪙 THINK token contracts | Enables micro-rewards to global contributors |
| 🎖️ Impact NFT system | Gives solvers verifiable, portable credentials |
| 🏛️ DAO governance | Puts platform decisions in the community's hands |
| 💸 Bounty escrow | Funds real implementations of winning solutions |
| 🔐 Security audits | Protects user funds and on-chain reputation |

### How to Contribute via Drips

1. Visit [drips.network](https://www.drips.network/app/projects/github/ThinknetCollective/mesh-contract)
2. Connect your wallet
3. Set a streaming amount (as low as $1/month)
4. Start streaming — funds flow continuously to contributors

> **Drips lets you support open-source sustainably.** No one-time donations — just continuous, fair compensation for ongoing work.

---

## ✨ Contract Architecture

```
contracts/
├── registry/
│   └── src/lib.rs             # Wave Program registration contract
├── escrow/
│   └── src/lib.rs             # Wave funding escrow contract
├── settlement/
│   └── src/lib.rs             # Wave completion settlement contract
scripts/
├── deploy.sh                  # Stellar testnet deployment script
└── seed_testnet.sh            # Testnet seeding script
│   └── src/lib.rs            # Wave Program registration contract
├── escrow/
│   └── src/lib.rs            # Wave escrow for fund management
├── settlement/
│   └── src/lib.rs            # Wave settlement and distribution
scripts/
├── deploy.sh                 # Automated testnet deployment
└── seed_testnet.sh           # Testnet data seeding
```

---

## 🚀 Quick Start

### Prerequisites

- Rust >= 1.70.0
- Stellar CLI (soroban-cli)
- Stellar SDK
- A wallet with Stellar testnet XLM (funded via Friendbot)
- Soroban CLI (install via `cargo install soroban-cli`)
- Stellar testnet account (get funded via [Friendbot](https://friendbot.stellar.org))

### Setup

```bash
# Clone the repo
git clone https://github.com/frankosakwe/mesh-contract.git
cd mesh-contract

# Install Rust toolchain for Soroban
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install soroban-cli

# Build contracts
cargo build --target wasm32-unknown-unknown --release

# Deploy to Stellar testnet (Linux/Mac/WSL)
chmod +x scripts/deploy.sh
./scripts/deploy.sh

# Seed testnet with sample data (Linux/Mac/WSL)
chmod +x scripts/seed_testnet.sh
./scripts/seed_testnet.sh
```

#### Windows Users

For Windows users, PowerShell scripts are also provided:

```powershell
# Deploy to Stellar testnet (PowerShell)
.\scripts\deploy.ps1

# Seed testnet with sample data (PowerShell)
.\scripts\seed_testnet.ps1
```

**Note:** Windows users experiencing build issues should use WSL (Windows Subsystem for Linux) or see [BUILD_ISSUES.md](BUILD_ISSUES.md) for workarounds.
cargo install soroban-cli

# Copy environment variables
cp .env.testnet.example .env.testnet
# Edit .env.testnet with your Stellar secret key

# Build contracts (native build)
cargo build

# Build WASM for deployment (requires soroban-cli)
cargo build --release --target wasm32-unknown-unknown

# Run tests (requires soroban-cli test features)
cargo test

# Deploy to Stellar testnet
./scripts/deploy.sh

# Seed testnet with sample data
./scripts/seed_testnet.sh
```

### Current Status

- ✅ Contracts compile successfully with `cargo build`
- ✅ Soroban SDK 20.5.0 compatibility issues resolved
- ⚠️ WASM compilation requires soroban-cli installation
- ⚠️ Unit tests have dependency issues with soroban-sdk test features
- 📝 Deployment scripts ready for use after soroban-cli installation

### Environment Variables

After deployment, contract IDs are automatically saved to `.env.testnet`:

```bash
# .env.testnet (auto-generated)
NETWORK=testnet
RPC_URL=https://soroban-testnet.stellar.org:443
DEPLOYER_ADDRESS=your_deployer_address
REGISTRY_CONTRACT_ID=deployed_registry_contract_id
ESCROW_CONTRACT_ID=deployed_escrow_contract_id
SETTLEMENT_CONTRACT_ID=deployed_settlement_contract_id
# .env.testnet
DEPLOYER_SECRET_KEY=your_stellar_secret_key
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org:443
```

---

## 🌍 Deployment Addresses

| Contract | Network | Address |
|---|---|---|
| Registry | Stellar Testnet | Deployed via `scripts/deploy.sh` |
| Escrow | Stellar Testnet | Deployed via `scripts/deploy.sh` |
| Settlement | Stellar Testnet | Deployed via `scripts/deploy.sh` |

> Contract IDs are saved to `.env.testnet` after deployment.
| Registry | Stellar Testnet | Run `./scripts/deploy.sh` to deploy |
| Escrow | Stellar Testnet | Run `./scripts/deploy.sh` to deploy |
| Settlement | Stellar Testnet | Run `./scripts/deploy.sh` to deploy |

> Testnet deployments are automated via deployment scripts.

---

## 🛠️ Tech Stack

| Layer | Technology |
|---|---|
| Smart contracts | Rust + Soroban SDK |
| Framework | Soroban CLI |
| Network | Stellar Testnet |
| Contract types | Registry, Escrow, Settlement |
| Deployment | Stellar CLI (soroban contract deploy) |
| Funding | Stellar Friendbot |

### Why Stellar (Soroban)?

| Feature | Stellar Soroban | Ethereum L1 |
|---|---|---|
| Gas fee | ~$0.0001 | ~$2–50 |
| Speed | ~5 seconds | ~12 seconds |
| Security | ✅ Stellar SCP | ✅ Native |
| Developer UX | ✅ Rust-based | ✅ EVM |
| Accessibility | ✅ Low cost for all | ❌ Expensive for small users |
| Framework | Soroban 20.0.0 |
| Testing | Soroban SDK testutils |
| Network | Stellar (Soroban) |
| Contract types | Registry, Escrow, Settlement |

### Why Stellar (Soroban)?

| Feature | Stellar | Ethereum L1 |
|---|---|---|
| Gas fee | ~$0.0001 | ~$2–50 |
| Speed | ~5 seconds | ~12 seconds |
| Security | ✅ SCP consensus | ✅ PoW/PoS |
| Developer UX | ✅ Rust-based | ✅ EVM |
| Accessibility | ✅ Very low cost | ❌ Expensive for small users |

---

## 🗺️ Roadmap

### ✅ Phase 1: Foundation (Q1 2026)
- [x] Repository setup & architecture design
- [x] Registry contract (Wave Program registration)
- [x] Escrow contract (Wave funding escrow)
- [x] Settlement contract (Wave completion settlement)
- [x] Testnet deployment scripts
- [x] Testnet seeding scripts

### 🚧 Phase 2: Testing & Integration (Q2 2026)
- [ ] Comprehensive contract tests
- [ ] Integration tests
- [x] Soroban contract development
- [x] Testnet deployment scripts
- [ ] Comprehensive test coverage
- [ ] Mainnet deployment

### 🚧 Phase 2: NFT & Governance (Q2 2026)
- [ ] ImpactNFT soulbound badges
- [ ] MeshDAO voting contracts
- [ ] Timelock controller
- [ ] Security audit
- [ ] Mainnet deployment

### 📋 Phase 3: Advanced Features (Q3 2026)
- [ ] Token integration
- [ ] Advanced governance features
- [ ] Frontend integration with thinkmesh-api
- [ ] Drips streaming integration

### 🌟 Phase 4: Ecosystem (Q4 2026)
- [ ] Cross-chain bridges
- [ ] Partner integrations
- [ ] Grant program smart contracts

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific contract tests
cargo test -p registry
cargo test -p escrow
cargo test -p settlement

# Run tests with output
cargo test -- --nocapture

# Run specific contract tests
cargo test -p registry
cargo test -p escrow
cargo test -p settlement
```

---

## 🚀 Stellar Testnet Deployment

### Prerequisites

1. **Install Rust and Soroban CLI**
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install soroban-cli
   ```

2. **Generate a Stellar Key Pair**
   ```bash
   soroban keys generate --network testnet
   ```
   This will generate a public key and secret key. Save the secret key securely.

3. **Fund Your Testnet Account**
   Visit [Stellar Friendbot](https://friendbot.stellar.org) and enter your public key to get testnet XLM.

### Deployment Steps

1. **Configure Environment**
   ```bash
   cp .env.testnet.example .env.testnet
   # Edit .env.testnet and set DEPLOYER_SECRET_KEY
   ```

2. **Build Contracts**
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

3. **Deploy to Testnet**
   ```bash
   ./scripts/deploy.sh
   ```
   This script will:
   - Deploy Registry contract
   - Deploy Escrow contract
   - Deploy Settlement contract
   - Initialize all contracts with proper wiring
   - Save contract IDs to `.env.testnet`

4. **Seed Test Data**
   ```bash
   ./scripts/seed_testnet.sh
   ```
   This script will:
   - Register a sample Wave Program
   - Open a Wave escrow
   - Fund the escrow
   - Create and approve a settlement proposal

### Verification

After deployment, you can verify the contracts are deployed correctly by checking the contract IDs in `.env.testnet`.

### Manual Contract Invocation

You can manually invoke contracts using the Soroban CLI:

```bash
# Read program details
soroban contract read \
  --id $REGISTRY_CONTRACT_ID \
  --fn get_program \
  --program_id your_program_id \
  --rpc-url https://soroban-testnet.stellar.org:443

# Read wave details
soroban contract read \
  --id $ESCROW_CONTRACT_ID \
  --fn get_wave \
  --wave_id your_wave_id \
  --rpc-url https://soroban-testnet.stellar.org:443
```

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development environment setup
- Coding standards (NatSpec documentation required)
- Pull request process
- Security disclosure policy

### 🎯 Open Issues — Apply Now

We maintain a list of open, well-scoped issues that contributors can pick up and implement:

| Label | Description |
|---|---|
| [`good first issue`](https://github.com/ThinknetCollective/mesh-contract/issues?q=is%3Aopen+label%3A%22good+first+issue%22) | Beginner-friendly tasks (Easy–Medium, 5–8 hrs) |
| [`smart-contract`](https://github.com/ThinknetCollective/mesh-contract/issues?q=is%3Aopen+label%3Asmart-contract) | All Solidity contract work |
| [`advanced`](https://github.com/ThinknetCollective/mesh-contract/issues?q=is%3Aopen+label%3Aadvanced) | Complex tasks for experienced contributors |

**To claim an issue:**
1. Comment `/apply` on the issue you want to work on
2. A maintainer will assign it to you
3. Fork, implement, and open a PR referencing the issue

**Quick ways to contribute:**
- 🐛 [Report bugs](https://github.com/ThinknetCollective/mesh-contract/issues/new?template=bug_report.yml)
- 💡 [Suggest features](https://github.com/ThinknetCollective/mesh-contract/issues/new?template=feature_request.yml)
- 🔧 [Propose a contract](https://github.com/ThinknetCollective/mesh-contract/issues/new?template=smart_contract.yml)
- 🔐 Responsible security disclosure: security@thinkmesh.io

---

## 🔐 Security

Smart contracts handle real funds. We take security seriously:

- All contracts use OpenZeppelin audited libraries
- NatSpec documentation required for all functions
- Comprehensive test coverage required (>95%)
- Bug bounty program: security@thinkmesh.io
- Responsible disclosure: [SECURITY.md](SECURITY.md)

> ⚠️ **Do not use unaudited contracts in production.** Mainnet deployment follows a full security audit.

---

## 🌟 Community & Links

- **ThinkMesh API**: [thinkmesh-api](https://github.com/ThinknetCollective/thinkmesh-api)
- **Mesh API**: [mesh-up_api](https://github.com/ThinknetCollective/mesh-up_api)
- **Discord**: https://discord.gg/thinkmesh
- **Twitter**: https://twitter.com/thinkmesh
- **Drips**: [Fund us on Drips](https://www.drips.network/app/projects/github/ThinknetCollective/mesh-contract)

---

## 📜 License

MIT License — see [LICENSE](LICENSE) for details.

Open source, forever.

---

<div align="center">

**💧 [Fund this project on Drips](https://www.drips.network/app/projects/github/ThinknetCollective/mesh-contract) — Wave 1 is LIVE**

🔗 Smart contracts • 🏆 On-chain reputation • 💸 Streaming rewards • 🌍 Real-world impact

Made with ❤️ by the ThinkMesh community

---

⭐ **Star this repo** if you believe in rewarding real-world problem solvers!

</div>