# Nozomi Examples

A collection of examples demonstrating how to use Nozomi for Solana blockchain interactions.

## Examples

### Jupiter Swap + Nozomi Tip
Demonstrates how to combine a Jupiter swap (SOL to USDC) with a SOL transfer in a single transaction.

## Prerequisites

- [Bun](https://bun.sh/) installed on your system
- Basic understanding of Solana transactions
- Wallet with SOL for testing

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd nozomi-examples
```

2. Install dependencies:
```bash
bun install
```

## Project Structure

```
nozomi-examples/
├── src/
│   ├── jupiter-swap/        # Jupiter swap + transfer example
│   │   ├── index.ts
│   │   └── README.md
├── package.json
└── README.md
```

## Running Examples

Each example can be run using Bun:

```bash
cd ./jup-swap-example
bun install
bun run index.ts
```

## Contributing

### Adding New Examples

1. Create a new directory under `src/`
2. Include:
   - Main implementation file (`index.ts`)
   - README with specific example documentation
   - Any additional helper files

### Guidelines

- Each example should be self-contained
- Include clear comments
- Add proper error handling
- Document any specific requirements

## Security Notes

1. Never commit private keys
2. Always verify addresses before transactions
3. Test with minimal amounts
4. Be careful with slippage settings in swaps

### General Issues
- Verify wallet has sufficient SOL
- Check RPC endpoint status