# Contributing to Mesh Contract

Thank you for your interest in contributing to Mesh Contract! This document outlines the process for contributing to the smart contract layer of the ThinkMesh ecosystem.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Smart Contract Standards](#smart-contract-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Security](#security)

---

## Code of Conduct

This project adheres to a [code of conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold these values:
- **Respectful collaboration** — All contributors deserve respect
- **Quality over quantity** — We value well-tested, documented code
- **Security first** — Smart contracts handle real funds; safety is non-negotiable

---

## Getting Started

### Prerequisites

- Node.js >= 18.0.0
- npm or yarn
- Git

### Setup

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/mesh-contract.git
cd mesh-contract

# Install dependencies
npm install

# Set up environment
cp .env.example .env
# Fill in .env with your testnet private key and RPC URLs

# Compile to verify setup
npm run compile

# Run tests to confirm everything works
npm test
```

---

## Development Workflow

### Branch Naming

```
feat/description-of-feature
fix/description-of-fix
docs/what-you-are-documenting
test/what-you-are-testing
chore/maintenance-task
```

### Making Changes

1. Create a branch from `main`
2. Write your code with NatSpec documentation
3. Write tests (coverage must stay above 95%)
4. Run the full test suite: `npm test`
5. Run static analysis: `npm run slither` (if available)
6. Open a Pull Request

---

## Smart Contract Standards

### NatSpec Documentation

All public and external functions **must** include NatSpec:

```solidity
/// @title ThinkToken
/// @notice ERC-20 reward token for ThinkMesh contributors
/// @dev Mintable by authorized roles only; implements ERC-20 + ERC-20Permit
contract ThinkToken {
    /// @notice Mint tokens to a contributor address
    /// @dev Only callable by MINTER_ROLE
    /// @param to The recipient address
    /// @param amount The amount of tokens to mint (18 decimals)
    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        _mint(to, amount);
    }
}
```

### Code Style

- Use [OpenZeppelin Contracts v5](https://docs.openzeppelin.com/contracts/5.x/) where possible
- Follow [Solidity Style Guide](https://docs.soliditylang.org/en/latest/style-guide.html)
- Use `^0.8.20` pragma minimum
- Prefer `custom errors` over `require` strings for gas efficiency
- Use `SafeERC20` for token transfers

### Security Patterns

- ✅ Use `ReentrancyGuard` for functions handling ETH/tokens
- ✅ Use `AccessControl` or `Ownable` for privileged functions
- ✅ Use `Pausable` for emergency stops
- ✅ Use OpenZeppelin's `Address.sendValue` not raw `.call`
- ❌ Never use `tx.origin` for authentication
- ❌ Never store sensitive data on-chain

---

## Testing Requirements

**Coverage must remain above 95%.** Run:

```bash
npm run coverage
```

### Test Structure

Each contract should have a corresponding test file:

```
test/
├── ThinkToken.test.ts
├── ReputationRegistry.test.ts
├── ImpactNFT.test.ts
├── MeshDAO.test.ts
└── BountyEscrow.test.ts
```

### Test Checklist

For every function, cover:
- ✅ Happy path (success)
- ✅ Access control (unauthorized callers revert)
- ✅ Edge cases (zero amounts, max values)
- ✅ Events emitted
- ✅ State changes verified

---

## Pull Request Process

1. **Fill out the PR template** completely
2. **Link the relevant issue** using `Closes #issue-number`
3. **Ensure CI passes** — all checks must be green
4. **Request review** from at least one core contributor
5. **Address all review comments** before merge

### PR Checklist

- [ ] NatSpec documentation on all new public/external functions
- [ ] Tests written and passing (`npm test`)
- [ ] Coverage stays above 95% (`npm run coverage`)
- [ ] No compiler warnings
- [ ] `.env.example` updated if new env vars added
- [ ] `CHANGELOG.md` updated (for non-trivial changes)

---

## Security

### Reporting Vulnerabilities

**Do NOT open public GitHub issues for security vulnerabilities.**

Email: **security@thinkmesh.io**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (optional)

We will respond within 48 hours and coordinate responsible disclosure.

### Bug Bounty

A formal bug bounty program is coming. For now, critical vulnerabilities discovered before mainnet launch will be acknowledged with THINK token grants upon launch.

---

## 💧 Support via Drips

If you find this project valuable, consider funding it through [Drips](https://www.drips.network/app/projects/github/ThinknetCollective/mesh-contract). Wave 1 is active — your streaming contribution helps us ship audited contracts faster.

---

Thank you for helping build the on-chain backbone of collaborative problem solving! 🚀
