# 🔗 Mesh Contract

**Smart contracts powering the ThinkMesh protocol — on-chain reputation, rewards, and governance.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solidity](https://img.shields.io/badge/Solidity-%5E0.8.20-blue)](https://soliditylang.org/)
[![Hardhat](https://img.shields.io/badge/Built%20with-Hardhat-yellow)](https://hardhat.org/)
[![Base Network](https://img.shields.io/badge/Base-Network-0052FF)](https://base.org/)
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
├── token/
│   └── ThinkToken.sol         # ERC-20 THINK reward token
├── reputation/
│   └── ReputationRegistry.sol # On-chain contribution scores
├── nft/
│   └── ImpactNFT.sol          # Soulbound impact badges (ERC-5192)
├── governance/
│   ├── MeshDAO.sol            # DAO voting & proposals
│   └── TimelockController.sol # Execution delay for proposals
├── bounty/
│   └── BountyEscrow.sol       # Escrow for funded challenges
└── interfaces/
    ├── IThinkToken.sol
    ├── IReputationRegistry.sol
    └── IBountyEscrow.sol
```

---

## 🚀 Quick Start

### Prerequisites

- Node.js >= 18.0.0
- npm or yarn
- A wallet with Base testnet ETH ([get some here](https://docs.base.org/docs/tools/network-faucets))

### Setup

```bash
# Clone the repo
git clone https://github.com/ThinknetCollective/mesh-contract.git
cd mesh-contract

# Install dependencies
npm install

# Copy environment variables
cp .env.example .env
# Edit .env with your private key and RPC URLs

# Compile contracts
npm run compile

# Run tests
npm test

# Deploy to Base testnet
npm run deploy:testnet
```

### Environment Variables

```bash
# .env.example
PRIVATE_KEY=your_wallet_private_key
BASE_MAINNET_RPC=https://mainnet.base.org
BASE_TESTNET_RPC=https://sepolia.base.org
BASESCAN_API_KEY=your_basescan_api_key
REPORT_GAS=true
```

---

## 🌍 Deployment Addresses

| Contract | Network | Address |
|---|---|---|
| ThinkToken | Base Mainnet | *Coming soon* |
| ReputationRegistry | Base Mainnet | *Coming soon* |
| ImpactNFT | Base Mainnet | *Coming soon* |
| MeshDAO | Base Mainnet | *Coming soon* |
| BountyEscrow | Base Mainnet | *Coming soon* |

> Testnet deployments available on Base Sepolia.

---

## 🛠️ Tech Stack

| Layer | Technology |
|---|---|
| Smart contracts | Solidity ^0.8.20 |
| Framework | Hardhat |
| Testing | Mocha + Chai + Waffle |
| OpenZeppelin | Contracts v5 |
| Network | Base (Coinbase L2) |
| Token standard | ERC-20 (THINK) |
| NFT standard | ERC-5192 (Soulbound) |
| Governance | OpenZeppelin Governor |

### Why Base (Not Ethereum Mainnet)?

| Feature | Base | Ethereum L1 |
|---|---|---|
| Gas fee | ~$0.001 | ~$2–50 |
| Speed | ~2 seconds | ~12 seconds |
| Security | ✅ Ethereum-backed | ✅ Native |
| Developer UX | ✅ EVM-compatible | ✅ EVM |
| Accessibility | ✅ Low cost for all | ❌ Expensive for small users |

---

## 🗺️ Roadmap

### ✅ Phase 1: Foundation (Q1 2026)
- [x] Repository setup & architecture design
- [ ] THINK token (ERC-20) deployment
- [ ] Basic reputation registry
- [ ] Testnet deployment

### 🚧 Phase 2: NFT & Governance (Q2 2026)
- [ ] ImpactNFT soulbound badges
- [ ] MeshDAO voting contracts
- [ ] Timelock controller
- [ ] Security audit

### 📋 Phase 3: Bounties & Rewards (Q3 2026)
- [ ] BountyEscrow contract
- [ ] Drips streaming integration
- [ ] Mainnet deployment
- [ ] Frontend integration with thinkmesh-api

### 🌟 Phase 4: Ecosystem (Q4 2026)
- [ ] Cross-chain bridges
- [ ] DAO-managed upgrade proxies
- [ ] Partner integrations
- [ ] Grant program smart contracts

---

## 🧪 Testing

```bash
# Run all tests
npm test

# Run tests with gas reporting
REPORT_GAS=true npm test

# Run coverage report
npm run coverage

# Run static analysis (Slither)
npm run slither
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