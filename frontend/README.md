# NEAR-ETH USDC Bridge

A seamless bridge application for transferring USDC from NEAR Protocol to Ethereum network. Works for Near and Sepolia testnet. Built with Near Chain Signatures.

## Features

- Direct USDC transfers from NEAR to Ethereum
- Real-time balance checking on both networks
- Seamless wallet integration
- User-friendly interface
- Automatic transaction status tracking

## Prerequisites

- Node.js 16.8 or later
- NEAR CLI installed (`npm install -g near-cli`)
- A NEAR account (testnet or mainnet)
- Some test USDC on NEAR network


## Installation

```bash
# Install dependencies
npm install
# or
yarn install

# Run the development server
npm run dev
# or
yarn dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

## Configuration

The application requires several contract configurations:

- NEAR USDC Contract
- Ethereum USDC Contract
- Bridge Contract


## Usage

1. Connect your NEAR wallet
2. Enter the amount of USDC to bridge
3. Provide the destination Ethereum address
4. Confirm the transaction in your NEAR wallet
5. Wait for the transaction to be processed on Ethereum

## Technical Details

- Built with Next.js 13+
- Uses `near-api-js` and `ethers.js` for blockchain interactions
- Implements Shadcn UI components
- Supports only testnet environment for now

## Learn More

- [NEAR Documentation](https://docs.near.org) - learn about NEAR Protocol
- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features

## Development

The project structure follows Next.js conventions:

```
/app                 # Next.js app directory
  /components        # React components
  /context          # Context providers
  /lib              # Utility functions
  /public           # Static assets
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[MIT](https://choosealicense.com/licenses/mit/)

## Support

For support, please raise an issue in the GitHub repository or contact the development team.