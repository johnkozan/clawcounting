# Currencies & Amounts

ClawCounting supports both fiat and crypto currencies with full precision.

## Amount Representation

All monetary amounts are stored as **i128 integers** in the smallest currency unit:
- No floating point -- zero rounding errors
- Maximum value: ±170,141,183,460,469,231,731,687,303,715,884,105,727
- Stored as 16-byte BLOBs in SQLite

The `asset_scale` on each currency defines decimal placement:

| Currency | Asset Scale | 1.0 stored as | $10.50 stored as |
|----------|------------|---------------|------------------|
| USD | 2 | `100` | `1050` |
| JPY | 0 | `1` | N/A |
| BTC | 8 | `100000000` | N/A |
| ETH | 18 | `1000000000000000000` | N/A |
| USDC | 6 | `1000000` | N/A |

**Formula**: `stored = display × 10^asset_scale`

### In API Requests

Amounts are sent as **string representations of the i128 value** in the smallest unit:

```json
{"debit_amount": "1050"}
```

Not `"10.50"` -- always the raw integer.

### In API Responses

Both forms are provided:

```json
{
  "debit_amount": "1050",
  "display_debit": "10.50"
}
```

## CAIP-19 Identifiers

Every currency has a unique [CAIP-19](https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-19.md) identifier -- a chain-agnostic standard that gives every asset a single canonical identifier:

| Type | Pattern | Example |
|------|---------|---------|
| Fiat (ISO 4217) | `swift:0/iso4217:<CODE>` | `swift:0/iso4217:USD` |
| Native coin (ETH) | `eip155:<chain>/slip44:<coin>` | `eip155:1/slip44:60` |
| ERC-20 token | `eip155:<chain>/erc20:<address>` | `eip155:1/erc20:0xa0b8...` |
| Bitcoin | `bip122:<genesis>/slip44:0` | `bip122:000000000019d6689c085ae165831e93/slip44:0` |

### Web UI

The **Add Currency** dialog in the web UI provides the easiest way to add currencies:

- **Fiat tab** -- searchable picker of all ISO 4217 currencies with country flags. Click to add.
- **Crypto tab** with three sub-tabs:
  - **Popular** -- searchable picker of native chains (BTC, ETH, SOL, etc.) and ~400 ERC-20 tokens from the Uniswap Default token list, with logos. Click to add.
  - **Import List** -- import tokens from any [Uniswap Token List](https://tokenlists.org/) standard JSON. Paste a URL or upload a file, preview the tokens with checkboxes, and bulk-add your selection.
  - **Custom** -- manual form for any currency not in the built-in lists.

Logos are resolved client-side: fiat currencies show country flags (derived from the ISO 4217 code), crypto currencies show token logos from the built-in data.

### CLI

#### Fiat Currencies

Use `create-fiat` with an ISO 4217 code -- ClawCounting auto-fills everything:

```bash
clawcounting currencies create-fiat USD --json
```

#### Crypto Currencies

Provide all fields manually:

```bash
clawcounting currencies create \
  --code ETH \
  --name "Ether" \
  --symbol "Ξ" \
  --asset-scale 18 \
  --type crypto \
  --caip19 "eip155:1/slip44:60" \
  --json
```

## Currency Constraints

- `code` is unique and immutable after creation
- `caip19_id` is unique and immutable after creation
- `asset_scale` is immutable (changing it would corrupt all existing balances)
- Only `name` and `symbol` can be updated

## Multi-Currency Rules

- All lines in a single journal entry must reference accounts with the **same currency**
- Cross-currency transactions require separate entries or a clearing account pattern
- Balance and report queries can filter by `currency_id`

### Cross-Currency Pattern

To record a currency exchange (e.g., sell USD for EUR), use two separate entries with a clearing account:

1. Entry in USD: Debit "Currency Exchange" clearing account, Credit USD Cash
2. Entry in EUR: Debit EUR Cash, Credit "Currency Exchange" clearing account

Exchange rate metadata can be stored in the journal entry's `metadata` JSON field.

## Multi-Chain Tokens

The same token on different chains is tracked as **separate currencies**. USDC on Ethereum and USDC on Solana are distinct currencies with distinct CAIP-19 IDs and distinct codes (e.g., "USDC-ETH" vs "USDC-SOL"). This is correct for accounting -- they have different settlement properties and may have different valuations.
