# Mesh Smart Contracts - Event Implementation

This directory contains the Soroban smart contracts for the Mesh platform with comprehensive on-chain event emissions for off-chain indexing.

## Overview

The Mesh platform uses three interconnected smart contracts to manage programs, waves, funding, and settlements:

- **Escrow Contract**: Manages fund deposits and releases
- **Settlement Contract**: Calculates and finalizes reward distributions
- **Registry Contract**: Maintains program and wave lifecycle state

All state-changing operations emit structured on-chain events that can be indexed by the Mesh platform's off-chain systems.

## Quick Start

### Prerequisites

- Rust 1.75 or later
- Soroban CLI 20.5 or compatible

### Build Contracts

```bash
# Build all contracts
cargo build --release

# Build specific contract
cd contracts/escrow && cargo build --release
cd contracts/settlement && cargo build --release
cd contracts/registry && cargo build --release
```

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific contract
cargo test --package mesh-escrow
cargo test --package mesh-settlement
cargo test --package mesh-registry

# Run integration tests
cargo test --test integration_tests

# Run tests with verbose output
cargo test -- --nocapture
```

## Project Structure

```
mesh-contract/
├── contracts/
│   ├── escrow/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          (escrow contract implementation)
│   │       └── tests.rs        (escrow event tests)
│   ├── settlement/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          (settlement contract implementation)
│   │       └── tests.rs        (settlement event tests)
│   └── registry/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs          (registry contract implementation)
│           └── tests.rs        (registry event tests)
├── tests/
│   └── integration_tests.rs    (cross-contract integration tests)
├── Cargo.toml                  (workspace configuration)
├── EVENTS.md                   (event implementation guide)
└── README.md                   (this file)
```

## Events Implementation

### Summary Table

| Contract | Event | Fields |
|----------|-------|--------|
| **Escrow** | `escrow_funded` | program_id (Symbol), amount (i128) |
| | `escrow_released` | wave_id (Symbol), recipient (Address), amount (i128) |
| **Settlement** | `settlement_settled` | wave_id (Symbol), total_points (i128), contributor_count (u32) |
| **Registry** | `registry_program_registered` | program_id (Symbol), name (Symbol), organizer (Address) |
| | `registry_wave_opened` | wave_id (Symbol), program_id (Symbol) |
| | `registry_wave_closed` | wave_id (Symbol), total_points (i128) |

### Event Naming Convention

All events use **snake_case** topic names with contract module prefixes:

```
escrow_*        - Escrow contract events
settlement_*    - Settlement contract events
registry_*      - Registry contract events
```

### Event Emission Pattern

All events are emitted via `env.events().publish()`:

```rust
env.events().publish(
    (Symbol::new(&env, "event_name"),),
    (field1, field2, field3),
);
```

## Escrow Contract

### Functions

#### `fund(env: Env, program_id: Symbol, amount: i128)`

Deposit funds into escrow for a program.

**Emits:** `escrow_funded(program_id, amount)`

**Validation:**
- Amount must be positive (> 0)

**Example:**
```rust
let client = EscrowContractClient::new(&env, &contract_id);
client.fund(&Symbol::new(&env, "program_001"), &1000);
```

#### `release(env: Env, wave_id: Symbol, recipient: Address, amount: i128)`

Release escrow funds to a recipient.

**Emits:** `escrow_released(wave_id, recipient, amount)`

**Validation:**
- Amount must be positive (> 0)
- Recipient must authorize the transaction

**Example:**
```rust
let client = EscrowContractClient::new(&env, &contract_id);
let recipient = Address::generate(&env);
client.release(&Symbol::new(&env, "wave_001"), &recipient, &500);
```

## Settlement Contract

### Functions

#### `settle(env: Env, wave_id: Symbol, total_points: i128, contributor_count: u32)`

Calculate and record points distribution for a wave.

**Emits:** `settlement_settled(wave_id, total_points, contributor_count)`

**Validation:**
- Total points must be positive (> 0)
- Contributor count must be > 0

**Example:**
```rust
let client = SettlementContractClient::new(&env, &contract_id);
client.settle(&Symbol::new(&env, "wave_001"), &1000, &20);
```

#### `finalize(env: Env, wave_id: Symbol, total_points: i128, contributor_count: u32)`

Finalize settlement and distribute rewards.

**Emits:** `settlement_settled(wave_id, total_points, contributor_count)`

**Validation:**
- Total points must be positive (> 0)
- Contributor count must be > 0

**Example:**
```rust
let client = SettlementContractClient::new(&env, &contract_id);
client.finalize(&Symbol::new(&env, "wave_001"), &1000, &20);
```

## Registry Contract

### Functions

#### `register_program(env: Env, program_id: Symbol, name: Symbol, organizer: Address)`

Register a new program.

**Emits:** `registry_program_registered(program_id, name, organizer)`

**Validation:**
- Organizer must authorize the transaction

**Example:**
```rust
let client = RegistryContractClient::new(&env, &contract_id);
let organizer = Address::generate(&env);
client.register_program(
    &Symbol::new(&env, "program_001"),
    &Symbol::new(&env, "Education Initiative"),
    &organizer
);
```

#### `open_wave(env: Env, wave_id: Symbol, program_id: Symbol)`

Open a new wave for a program.

**Emits:** `registry_wave_opened(wave_id, program_id)`

**Validation:**
- Wave ID must not be empty
- Program ID must not be empty

**Example:**
```rust
let client = RegistryContractClient::new(&env, &contract_id);
client.open_wave(
    &Symbol::new(&env, "wave_001"),
    &Symbol::new(&env, "program_001")
);
```

#### `close_wave(env: Env, wave_id: Symbol, total_points: i128)`

Close a wave.

**Emits:** `registry_wave_closed(wave_id, total_points)`

**Validation:**
- Total points cannot be negative (>= 0)

**Example:**
```rust
let client = RegistryContractClient::new(&env, &contract_id);
client.close_wave(&Symbol::new(&env, "wave_001"), &5000);
```

#### `update_program(env: Env, program_id: Symbol, name: Symbol, organizer: Address)`

Update program information.

**Emits:** `registry_program_registered(program_id, name, organizer)`

**Validation:**
- Organizer must authorize the transaction

**Example:**
```rust
let client = RegistryContractClient::new(&env, &contract_id);
client.update_program(
    &Symbol::new(&env, "program_001"),
    &Symbol::new(&env, "Updated Program Name"),
    &organizer
);
```

## Testing

All contracts include comprehensive unit tests that verify:

✅ Events are emitted for every state-changing operation
✅ Event topics use correct snake_case naming
✅ Event data contains all required fields with correct values
✅ Validation constraints are properly enforced
✅ Multiple operations correctly emit multiple events

### Run Specific Test

```bash
# Test a specific function
cargo test test_escrow_fund_emits_event

# Run tests with output
cargo test -- --nocapture

# Run tests with specific number of threads
cargo test -- --test-threads=1
```

## Off-Chain Indexing

Events emitted by these contracts are designed for off-chain indexing by the Mesh platform:

1. **Escrow Indexing**: Track fund flows through `escrow_funded` and `escrow_released` events
2. **Settlement Indexing**: Record point distributions and contributor settlements via `settlement_settled` events
3. **Registry Indexing**: Maintain program and wave state via `registry_*` events

### Event Retrieval (Test Environment)

```rust
let env = Env::default();
// ... perform contract operations ...

// Retrieve all emitted events
let events = env.events().all();

// Iterate through events
for event in events {
    let topics = &event.topics;
    let data = &event.data;
    // ... process event ...
}
```

## Validation & Error Handling

All contracts enforce strict validation:

**Escrow Contract:**
- ✓ Amounts must be positive
- ✓ Recipients must authorize releases

**Settlement Contract:**
- ✓ Points must be positive
- ✓ Contributor count must be > 0

**Registry Contract:**
- ✓ IDs must not be empty
- ✓ Points cannot be negative
- ✓ Organizers must authorize

Contract execution will panic with descriptive error messages if validation fails.

## Acceptance Criteria ✅

✅ All three contracts implemented with Soroban SDK
✅ Structured events emitted for every state-changing operation
✅ Event topics use consistent snake_case naming convention
✅ All required event fields included with correct types
✅ Comprehensive test suite verifies event emission
✅ Events readable via `env.events()` in tests
✅ Ready for off-chain indexing by Mesh platform

## Compilation

```bash
# Release build
cargo build --release

# Generate WASM binaries
cargo build --release --target wasm32-unknown-unknown
```

## Documentation

For detailed event specifications and off-chain indexing guide, see [EVENTS.md](EVENTS.md).

## Contributing

When adding new functions to contracts:

1. Emit appropriate events using snake_case topics
2. Include all state-changing information in event data
3. Add unit tests verifying event emission
4. Update EVENTS.md with new event specifications

## License

MIT License - See LICENSE file for details
