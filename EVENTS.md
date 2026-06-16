# Smart Contract Event Implementation Guide

## Overview

This document describes the on-chain events emitted by the Mesh contract suite for off-chain indexing by the Mesh platform. All events follow consistent naming conventions and are emitted using Soroban's `env.events().publish()` API.

## Event Naming Convention

- All event topics use **snake_case** symbols
- Event topics are prefixed with the contract module name:
  - `escrow_*` - Escrow contract events
  - `settlement_*` - Settlement contract events
  - `registry_*` - Registry contract events

## Escrow Contract Events

### Event: `escrow_funded`

Emitted when funds are deposited into escrow for a program.

**Topic:** `escrow_funded` (Symbol)

**Data Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `program_id` | Symbol | Unique identifier for the program |
| `amount` | i128 | Amount of funds deposited (must be positive) |

**Example:**
```rust
env.events().publish(
    (Symbol::new(&env, "escrow_funded"),),
    (program_id, amount),
);
```

**Usage:**
```rust
let client = EscrowContractClient::new(&env, &contract_id);
client.fund(&Symbol::new(&env, "program_001"), &1000);
```

### Event: `escrow_released`

Emitted when escrow funds are released to a recipient for a wave.

**Topic:** `escrow_released` (Symbol)

**Data Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `wave_id` | Symbol | Unique identifier for the wave |
| `recipient` | Address | Recipient address receiving the funds |
| `amount` | i128 | Amount released from escrow (must be positive) |

**Example:**
```rust
env.events().publish(
    (Symbol::new(&env, "escrow_released"),),
    (wave_id, recipient, amount),
);
```

**Usage:**
```rust
let client = EscrowContractClient::new(&env, &contract_id);
let recipient = Address::generate(&env);
client.release(&Symbol::new(&env, "wave_001"), &recipient, &500);
```

## Settlement Contract Events

### Event: `settlement_settled`

Emitted when a wave settlement is finalized, calculating points distribution and confirming contributors.

**Topic:** `settlement_settled` (Symbol)

**Data Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `wave_id` | Symbol | Unique identifier for the settled wave |
| `total_points` | i128 | Total points distributed in this wave (must be positive) |
| `contributor_count` | u32 | Number of contributors in this wave (must be > 0) |

**Example:**
```rust
env.events().publish(
    (Symbol::new(&env, "settlement_settled"),),
    (wave_id, total_points, contributor_count),
);
```

**Usage:**
```rust
let client = SettlementContractClient::new(&env, &contract_id);
client.settle(&Symbol::new(&env, "wave_001"), &5000, &20);
```

## Registry Contract Events

### Event: `registry_program_registered`

Emitted when a new program is registered or updated in the registry.

**Topic:** `registry_program_registered` (Symbol)

**Data Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `program_id` | Symbol | Unique identifier for the program |
| `name` | Symbol | Human-readable name of the program |
| `organizer` | Address | Address of the program organizer |

**Example:**
```rust
env.events().publish(
    (Symbol::new(&env, "registry_program_registered"),),
    (program_id, name, organizer),
);
```

**Usage:**
```rust
let client = RegistryContractClient::new(&env, &contract_id);
let organizer = Address::generate(&env);
client.register_program(
    &Symbol::new(&env, "program_001"),
    &Symbol::new(&env, "Education Initiative"),
    &organizer
);
```

### Event: `registry_wave_opened`

Emitted when a new wave is opened for a program.

**Topic:** `registry_wave_opened` (Symbol)

**Data Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `wave_id` | Symbol | Unique identifier for the wave |
| `program_id` | Symbol | Program identifier this wave belongs to |

**Example:**
```rust
env.events().publish(
    (Symbol::new(&env, "registry_wave_opened"),),
    (wave_id, program_id),
);
```

**Usage:**
```rust
let client = RegistryContractClient::new(&env, &contract_id);
client.open_wave(
    &Symbol::new(&env, "wave_001"),
    &Symbol::new(&env, "program_001")
);
```

### Event: `registry_wave_closed`

Emitted when a wave is closed and finalized.

**Topic:** `registry_wave_closed` (Symbol)

**Data Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `wave_id` | Symbol | Unique identifier for the closed wave |
| `total_points` | i128 | Total points distributed in this wave (can be >= 0) |

**Example:**
```rust
env.events().publish(
    (Symbol::new(&env, "registry_wave_closed"),),
    (wave_id, total_points),
);
```

**Usage:**
```rust
let client = RegistryContractClient::new(&env, &contract_id);
client.close_wave(
    &Symbol::new(&env, "wave_001"),
    &5000
);
```

## Event Access in Tests

All events are emitted via `env.events().publish()` and can be accessed in tests using `env.events().all()`:

```rust
#[test]
fn test_event_emission() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    // Perform an operation that emits an event
    client.some_function();

    // Access all emitted events
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;
    
    // Verify event topic
    if let soroban_sdk::Val::Symbol(s) = &event_topics[0] {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes()).unwrap();
        assert_eq!(topic_str, "expected_event_name");
    }
    
    // Access event data
    let event_data = &event.data;
    // ... verify event data
}
```

## Off-Chain Indexing

The Mesh platform indexes these events to:

1. **Track funding flows:** Monitor `escrow_funded` and `escrow_released` events to track fund movement
2. **Calculate settlements:** Use `settlement_settled` events to compute contributor rewards and points
3. **Maintain registries:** Monitor `registry_program_registered`, `registry_wave_opened`, and `registry_wave_closed` events to keep an up-to-date program and wave registry
4. **Generate reports:** Create dashboards and analytics based on event data

### Event Indexing Example

```typescript
// Pseudo-code for off-chain indexer
async function indexEscrowEvents() {
  const events = await fetchContractEvents('escrow_funded');
  for (const event of events) {
    const { program_id, amount } = event.data;
    await db.updateProgramFunding(program_id, amount);
  }
}

async function indexSettlementEvents() {
  const events = await fetchContractEvents('settlement_settled');
  for (const event of events) {
    const { wave_id, total_points, contributor_count } = event.data;
    await db.finalizWaveSettlement(wave_id, total_points, contributor_count);
  }
}
```

## Running Tests

To verify that all events are properly emitted, run the test suite:

```bash
# Test individual contracts
cd contracts/escrow
cargo test

cd ../settlement
cargo test

cd ../registry
cargo test

# Run all tests from workspace root
cargo test --workspace
```

Each contract includes comprehensive tests that verify:
- ✅ Events are emitted for every state-changing operation
- ✅ Event topics use correct snake_case naming
- ✅ Event data contains all required fields
- ✅ Validation constraints are enforced
- ✅ Multiple operations emit multiple events

## Validation & Error Handling

All functions enforce the following validation rules:

### Escrow Contract
- `fund()`: Amount must be positive (> 0)
- `release()`: Amount must be positive (> 0); recipient must authorize

### Settlement Contract
- `settle()`: Total points must be positive (> 0); contributor count must be > 0
- `finalize()`: Total points must be positive (> 0); contributor count must be > 0

### Registry Contract
- `open_wave()`: Wave ID and program ID must not be empty
- `close_wave()`: Total points cannot be negative (>= 0)
- `register_program()`: Organizer must authorize

Invalid inputs will cause contract execution to panic with descriptive error messages.

## Architecture

All contracts follow Soroban best practices:

```
contracts/
├── escrow/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs (contract implementation)
│       └── tests.rs (event tests)
├── settlement/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs (contract implementation)
│       └── tests.rs (event tests)
└── registry/
    ├── Cargo.toml
    └── src/
        ├── lib.rs (contract implementation)
        └── tests.rs (event tests)
```

## Summary

| Contract | Event | Fields |
|----------|-------|--------|
| **Escrow** | `escrow_funded` | program_id, amount |
| | `escrow_released` | wave_id, recipient, amount |
| **Settlement** | `settlement_settled` | wave_id, total_points, contributor_count |
| **Registry** | `registry_program_registered` | program_id, name, organizer |
| | `registry_wave_opened` | wave_id, program_id |
| | `registry_wave_closed` | wave_id, total_points |

All events are critical for the Mesh platform's off-chain indexing and must be emitted reliably for every state-changing operation.
