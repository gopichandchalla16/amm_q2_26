# Turbin3 Q3 2026 - Week 3 Assignment: Automated Market Maker (AMM)

Implementation of the Turbin3 Builders Week 3 Assignment (AMMs) on Solana using the Anchor framework.

This repository implements a Constant Product Automated Market Maker (`X * Y = K`). It supports liquidity provision, token swaps with slippage protection, protocol fee collection into a dedicated Treasury (PDA), and a LiteSVM test suite covering all instructions.

## Architecture and Account Structure

The program uses PDAs to hold funds and enforce permissions.

**Config (PDA)**  
`seeds = [b"config", seed.to_le_bytes()]`

Stores:
- `mint_x`, `mint_y`
- `fee` (basis points)
- `locked`
- `authority`
- `config_bump`, `lp_bump`

**Other accounts**
- `mint_lp` – LP token mint, authority is the config PDA  
  `seeds = [b"lp", config]`
- `vault_x` / `vault_y` – pool reserves (ATAs owned by config)
- `treasury_authority` – PDA  
  `seeds = [b"treasury", config]`
- `treasury_x` / `treasury_y` – protocol fee accounts (ATAs owned by treasury_authority)

## Instructions

### 1. initialize
Creates the Config PDA, LP mint, vault ATAs, and treasury ATAs. Sets the fee and optional authority.

### 2. deposit
User deposits token X and token Y. The program mints LP tokens proportional to the contribution. First deposit sets the initial ratio using the amounts supplied by the user. Later deposits use the constant product curve to keep the pool balanced.

### 3. withdraw
User burns LP tokens and receives the proportional share of token X and token Y from the vaults. Minimum output amounts protect against slippage.

### 4. swap
User swaps one token for the other using the constant product curve. The fee is applied during the swap calculation. A portion of the fee is transferred from the relevant vault into the matching treasury token account. Slippage is protected with `min_amount_out`. Zero amount swaps are rejected.

## Fee and Treasury Design

- Fee is stored on Config in basis points (example: 30 = 0.30%)
- On every swap a portion of the fee is moved from the vault into the treasury
- Treasury accounts are owned by a PDA so fee handling stays under program control
- Protocol fees are kept separate from the liquidity that belongs to LPs

## Tests (LiteSVM)

```bash
cargo test --test tests -- --nocapture
```

| Test | What it verifies |
|------|------------------|
| `test_initialize` | Config, vaults, treasury and LP mint are created correctly |
| `test_deposit` | Liquidity can be added and LP tokens are minted |
| `test_withdraw` | LP tokens can be burned and underlying tokens are returned |
| `test_swap_x_for_y` | Swap from X to Y works and fee path is exercised |
| `test_swap_y_for_x` | Swap from Y to X works |
| `test_swap_rejects_zero_amount` | Zero amount swap is rejected |

## Build

```bash
anchor build
```

## Test Result Screenshot

![All 6 tests passing](https://raw.githubusercontent.com/gopichandchalla16/amm_q2_26/main/amm_tests_passing.png)

## Notes

- Curve math uses the `constant_product_curve` library
- Treasury is intentionally separated from the pool vaults
- Tests cover the main path for every instruction plus a negative case for zero amount

Gopichand  
Turbin3 Builders Cohort Q3 2026
```
