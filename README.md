# Turbin3 Q3 2026 - Week 3 Assignment: Automated Market Maker (AMM)

Solana Anchor implementation of a Constant Product Automated Market Maker (`X * Y = K`) for the Turbin3 Builders Cohort Week 3 assignment.

The program supports pool creation, liquidity provision, liquidity withdrawal, token swaps with slippage protection, and protocol fee routing into a dedicated Treasury PDA.

**Author:** Gopichand  
**Cohort:** Turbin3 Builders Cohort Q3 2026  
**Program:** `amm-video`

## What this repository contains

This submission covers the three required tasks:

1. Full AMM program with `initialize`, `deposit`, `withdraw`, and `swap`
2. Configurable swap fee plus treasury accounts owned by a PDA
3. LiteSVM tests covering every instruction and one negative case

Optional extension items (CPMM without the library, downtime mitigation write-up) were not required for this submission.

## Architecture

The program is stateless. All mutable state lives in accounts. PDAs are used both for custody and for signing.

### Config PDA

```
seeds = [b"config", seed.to_le_bytes()]
```

Fields stored on Config:

| Field | Type | Purpose |
|-------|------|---------|
| `seed` | `u64` | Allows multiple independent pools for the same token pair |
| `authority` | `Option<Pubkey>` | Optional admin key |
| `mint_x` | `Pubkey` | Token X mint |
| `mint_y` | `Pubkey` | Token Y mint |
| `fee` | `u16` | Swap fee in basis points (30 = 0.30%) |
| `locked` | `bool` | Emergency pause flag |
| `config_bump` | `u8` | Canonical bump for Config |
| `lp_bump` | `u8` | Canonical bump for LP mint |

### Other accounts

| Account | Ownership / Seeds | Role |
|---------|-------------------|------|
| `mint_lp` | PDA `["lp", config]` | LP token mint, mint authority is Config |
| `vault_x` | ATA of Config for mint_x | Pool reserve for token X |
| `vault_y` | ATA of Config for mint_y | Pool reserve for token Y |
| `treasury_authority` | PDA `["treasury", config]` | Authority over treasury token accounts |
| `treasury_x` | ATA of treasury_authority for mint_x | Protocol fee account for token X |
| `treasury_y` | ATA of treasury_authority for mint_y | Protocol fee account for token Y |

## Instructions

### initialize

Creates the full pool surface in one transaction:

- Config PDA
- LP mint (6 decimals, authority = Config)
- `vault_x` and `vault_y`
- `treasury_authority` PDA
- `treasury_x` and `treasury_y`

Parameters: `seed`, `fee`, optional `authority`.

### deposit

Adds liquidity and mints LP tokens.

- First deposit uses the exact `max_x` / `max_y` amounts supplied by the user and sets the initial price ratio
- Later deposits use the constant product curve (`xy_deposit_amounts_from_l`) so the user must deposit in the current pool ratio
- Slippage is enforced with `max_x` and `max_y`
- Pool must not be locked

### withdraw

Burns LP tokens and returns the proportional share of both reserves.

- Amounts are computed with `xy_withdraw_amounts_from_l`
- User sets `min_x` and `min_y` for slippage protection
- Pool must not be locked

### swap

Swaps one token for the other.

Flow:

1. Build the constant product curve from current vault balances, LP supply, and fee
2. Compute deposit and withdraw amounts with slippage check (`min_amount_out`)
3. Transfer input tokens from user into the matching vault
4. Transfer output tokens from the other vault to the user
5. Calculate fee portion and move it from the input vault into the matching treasury account

Fee routing detail:

- Fee is read from Config (basis points)
- Half of the computed fee amount is sent to treasury (`fee_amount / 2`)
- Transfer is signed by the Config PDA

Zero amount swaps are rejected with `AmmError::InvalidAmount`. Locked pools reject swaps with `AmmError::PoolLocked`.

## Fee and Treasury design

- Fee is set once at `initialize` and stored on Config
- The curve library applies the fee inside the swap math so LPs still capture part of the spread
- An additional portion is explicitly transferred into `treasury_x` or `treasury_y`
- Treasury token accounts are owned by `treasury_authority` (PDA), not by a normal wallet
- This separates protocol revenue from LP-owned reserves

## Tests

All tests run against the compiled SBF binary using LiteSVM (no external validator).

```bash
cargo test --test tests -- --nocapture
```

| Test | Coverage |
|------|----------|
| `test_initialize` | Config, LP mint, vaults, and treasury accounts are created |
| `test_deposit` | Liquidity addition and LP minting after initialize |
| `test_withdraw` | LP burn and proportional redemption after deposit |
| `test_swap_x_for_y` | Swap direction X → Y including fee path |
| `test_swap_y_for_x` | Swap direction Y → X |
| `test_swap_rejects_zero_amount` | Zero input is rejected |

## Build

```bash
anchor build
```

## Test Result Screenshot

![All 6 tests passing](https://raw.githubusercontent.com/gopichandchalla16/amm_q2_26/main/amm_tests_passing.png)

## Notes

- Curve math is provided by the `constant_product_curve` crate
- All vault and treasury transfers that move pool funds are signed with Config PDA seeds
- Tests cover every required instruction and one failure path
- This repository satisfies the Week 3 mandatory requirements: AMM program, fees + treasury, and tests with screenshot

Gopichand  
Turbin3 Builders Cohort Q3 2026
