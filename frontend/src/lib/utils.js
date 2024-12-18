import { clsx } from "clsx";
import { twMerge } from "tailwind-merge"
const { ethers } = require('ethers');

export function cn(...inputs) {
  return twMerge(clsx(inputs));
}


export async function getBridgeRequest(ethReciever){
  const usdcEthTreasury = "0xd6cEefFa721575c53181346b6cC49647167085c7"; // Need this for finding out nonce. This is controlled by Near smart contract

  const {maxPriorityFeePerGas, maxFeePerGas} = await getGasDetails();
  const eth_nonce = await getNonce(usdcEthTreasury);
  
  const networkDetails = {
    max_priority_fee_per_gas: Number(maxPriorityFeePerGas), 
    max_fee_per_gas: Number(maxFeePerGas),
    gas_limit: 60000,
    chain_id: 11155111, // Sepolia testnet
    eth_nonce: eth_nonce, // Replace with the actual nonce
  };
  console.log("Network details:", networkDetails);

  const bridgeRequest ={
    eth_address: ethReciever,
    network_details: networkDetails,
  }
  return bridgeRequest;
}

async function getGasDetails(){
  let provider;
  let maxPriorityFeePerGas, maxFeePerGas;
  if (ethers.providers && ethers.providers.JsonRpcProvider) {
      // ethers v5
      provider = new ethers.providers.JsonRpcProvider('https://rpc.sepolia.org/');
  } else if (ethers.JsonRpcProvider) {
      // ethers v6
      provider = new ethers.JsonRpcProvider('https://rpc.sepolia.org/');
  } else {
      throw new Error('Unable to create JsonRpcProvider. Check ethers version and import.');
  }
  const feeData = await provider.getFeeData();
  //console.log('Fee data:', feeData);
  maxPriorityFeePerGas = feeData.maxPriorityFeePerGas;
  maxFeePerGas = feeData.maxFeePerGas;
  return {maxPriorityFeePerGas, maxFeePerGas};
}

async function getNonce(address){
  let provider;
  if (ethers.providers && ethers.providers.JsonRpcProvider) {
      // ethers v5
      provider = new ethers.providers.JsonRpcProvider('https://rpc.sepolia.org/');
  } else if (ethers.JsonRpcProvider) {
      // ethers v6
      provider = new ethers.JsonRpcProvider('https://rpc.sepolia.org/');
  } else {
      throw new Error('Unable to create JsonRpcProvider. Check ethers version and import.');
  }
  const nonce = await provider.getTransactionCount(address);
  return nonce;
}

export async function sendRawTransaction(signedTxBytes) {
  console.log('Received signedTxBytes:', signedTxBytes);
  
  // Convert Vec<u8> to hexadecimal string
  const signedTxHex = '0x' + Buffer.from(signedTxBytes).toString('hex');
  console.log('Converted to hex:', signedTxHex);

  // Set up the Ethereum provider
  let provider;
  if (ethers.providers && ethers.providers.JsonRpcProvider) {
      // ethers v5
      provider = new ethers.providers.JsonRpcProvider('https://rpc.sepolia.org/');
  } else if (ethers.JsonRpcProvider) {
      // ethers v6
      provider = new ethers.JsonRpcProvider('https://rpc.sepolia.org/');
  } else {
      throw new Error('Unable to create JsonRpcProvider. Check ethers version and import.');
  }
  console.log('Provider set up');

  try {
      console.log('Attempting to send transaction...');
      // Send the raw transaction
      const tx = await provider.broadcastTransaction(signedTxHex);
      console.log('Transaction sent successfully. Hash:', tx.hash);
      
      // Wait for the transaction to be mined
      console.log('Waiting for transaction to be mined...');
      const receipt = await tx.wait();
      console.log('Transaction mined. Block number:', receipt.blockNumber);

      return tx.hash;
  } catch (error) {
      console.error('Failed to send transaction. Error details:', error);
      if (error.reason) console.error('Error reason:', error.reason);
      if (error.code) console.error('Error code:', error.code);
      if (error.transaction) console.error('Error transaction:', error.transaction);
      throw error;
  }
}
