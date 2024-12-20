import { useState, useEffect, useContext } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { ArrowLeftRight, Loader2 } from "lucide-react"
import { NearContext } from "@/context"
import { USDC_CONTRACT_ID_NEAR, USDC_CONTRACT_ID_ETH, BRIDGE_CONTRACT_ID } from "@/config"
import { getBridgeRequest, sendRawTransaction, getProvider } from "../lib/utils"
const { ethers } = require('ethers');


export default function LandingPage() {
  const { signedAccountId, wallet } = useContext(NearContext)
  const [action, setAction] = useState(() => {})
  const [label, setLabel] = useState("Loading...")
  const [amount, setAmount] = useState("")
  const [ethAddress, setEthAddress] = useState("")
  const [isLoading, setIsLoading] = useState(false)
  const [txHash, setTxHash] = useState(null)
  const [nearTxHash, setNearTxHash] = useState(null)
  const [isConfirmed, setIsConfirmed] = useState(false)
  const [error, setError] = useState(null)
  const [balances, setBalances] = useState({
    nearBalance: null,
    ethBalance: null
  })

  useEffect(() => {
    if (!wallet) return

    if (signedAccountId) {
      setAction(() => wallet.signOut)
      setLabel(`Logout ${signedAccountId}`)
      fetchBalances() // Fetch balances when account is connected
    } else {
      setAction(() => wallet.signIn)
      setLabel("Connect Wallet")
    }
  }, [signedAccountId, wallet])

  // Fetch USDC balances for both chains
  const fetchBalances = async () => {
    try {
      // Fetch NEAR USDC balance
      const nearBalance = await wallet.viewMethod({
        contractId: USDC_CONTRACT_ID_NEAR,
        method: "ft_balance_of",
        args: { account_id: signedAccountId }
      })

      console.log("NEAR Balance:", nearBalance)

      // Fetch ETH USDC balance if ethAddress is set
      let ethBalance = null
      if (ethAddress) {
        const provider = new ethers.JsonRpcProvider('https://rpc.sepolia.org/')
        const usdcContract = new ethers.Contract(USDC_CONTRACT_ID_ETH, ['function balanceOf(address) view returns (uint256)'], provider)
        ethBalance = await usdcContract.balanceOf(ethAddress)
      }

      console.log("ETH Balance:", ethBalance)

      setBalances({
        nearBalance: nearBalance ? (Number(nearBalance) / 1e6).toFixed(2) : null,
        ethBalance: ethBalance ? (Number(ethBalance) / 1e6).toFixed(2) : null
      })
    } catch (err) {
      console.error("Error fetching balances:", err)
    }
  }

  // Automatically fetch balances when ethAddress changes
  useEffect(() => {
    if (signedAccountId && ethAddress) {
      fetchBalances()
    }
  }, [ethAddress, signedAccountId])

    // Check for returning from wallet and complete transaction
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search)
    const transactionHashes = urlParams.get('transactionHashes')
    
    if (transactionHashes) {
      completeTransaction()
      // Clean up URL
      window.history.replaceState({}, '', window.location.pathname)
    }
  }, [])

  const completeTransaction = async () => {
    try {
      setIsLoading(true)
      console.log("In complete Transaction.")
      
      // Get signed transaction from bridge contract
      const signedTxBytes = await wallet.viewMethod({
        contractId: BRIDGE_CONTRACT_ID,
        method: "get_latest_signed_tx"
      })

      // Send to Ethereum and get both hash and confirmation promise
      const { hash, confirmation } = await sendRawTransaction(signedTxBytes)
      setTxHash(hash)
      setIsConfirmed(false)
      
      // Wait for confirmation in background
      confirmation.then(async () => {
        setIsConfirmed(true)
        await fetchBalances()
      }).catch(err => {
        console.error("Error during confirmation:", err)
        setError("Transaction sent but confirmation failed. Please check explorer for status.")
      })
    } catch (err) {
      console.error("Failed to complete transaction:", err)
      setError(err.message || "Failed to complete transaction. Please try again.")
    } finally {
      setIsLoading(false)
    }
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    setIsLoading(true)
    setError(null)
    setTxHash(null)
    setNearTxHash(null)

    try {
      const formattedEthAddress = ethAddress.startsWith('0x') 
      ? ethAddress.slice(2) 
      : ethAddress

      const bridgeRequest = await getBridgeRequest(formattedEthAddress)
      //console.log("Bridge request:", bridgeRequest)
      
      let nearResult;
      try {
        //Call NEAR contract
        nearResult = await wallet.callMethod({
          contractId: USDC_CONTRACT_ID_NEAR,
          method: "ft_transfer_call",
          args: { 
            amount: (Number(amount) * 1e6).toString(), // Convert to USDC decimals
            receiver_id: BRIDGE_CONTRACT_ID,
            msg: JSON.stringify(bridgeRequest)
          },
          deposit: "1",
        })
        
        // Set NEAR transaction hash for normal (non-timeout) case
        if (nearResult?.transaction?.hash) {
          setNearTxHash(nearResult.transaction.hash)
        }
      } catch (e) {
        // Handle timeout case where transaction was submitted but RPC timed out
        if (e.context?.transactionHash) {
          const timeoutHash = e.context.transactionHash;
          console.log(`Transaction submitted but timed out. Hash: ${timeoutHash}`);
          setNearTxHash(timeoutHash);
          
          // Wait for 15 seconds
          await new Promise(resolve => setTimeout(resolve, 15000));
          
          // Check transaction status
          nearResult = await wallet.getTransactionResult(timeoutHash);
          console.log("Transaction completed after timeout:", nearResult);
        } else {
          throw e; // Re-throw if it's not a timeout error
        }
      }

      // If we got here, the NEAR transaction was successful (either directly or after timeout)
      // Now we can complete the Ethereum part
      await completeTransaction();
    } catch (err) {
      console.error("Transaction failed:", err)
      setError(err.message || "Transaction failed. Please try again.")
    } finally {
      setIsLoading(false)
    }
  }

  const BalanceDisplay = ({ label, balance }) => (
    balance !== null && (
      <div className="text-sm text-blue-600 mt-1">
        {label}: {balance} USDC
      </div>
    )
  )

  return (
    <div className="min-h-screen flex flex-col bg-gradient-to-br from-blue-400 via-blue-500 to-blue-600 text-white">
      <div className="container mx-auto px-4 py-8 relative z-10 flex-grow">
        <header className="flex justify-between items-center mb-12">
          <h1 className="text-3xl font-bold">DirectTransfer</h1>
          <Button 
            variant="outline" 
            className="bg-white text-blue-600 rounded-full hover:bg-blue-100 transition-colors duration-300" 
            onClick={action}
          >
            {label}
          </Button>
        </header>

        <main className="grid md:grid-cols-2 gap-12 items-start">
        <div className="space-y-6">
             <h2 className="text-5xl font-bold leading-tight">Seamless USDC Transfers from NEAR to Ethereum</h2>
             <p className="text-xl">A direct USDC transfer service that lets you send USDC from Near to any Ethereum address without traditional bridging.</p>
             <p className="text-xl">When you send USDC to our Near contract, it uses Chain Signatures to initiate a USDC transfer on Ethereum through its controlled address.</p>
             <ul className="space-y-2">
               <li className="flex items-center">
                 <span className="bg-white text-blue-600 rounded-full w-6 h-6 flex items-center justify-center mr-3">✓</span>
                 <p>No complicated steps or long waiting times</p>
               </li>
               <li className="flex items-center">
                 <span className="bg-white text-blue-600 rounded-full w-6 h-6 flex items-center justify-center mr-3">✓</span>
                 <p>Secure and transparent bridging process</p>
               </li>
             </ul>
           </div>
          
          <Card className="bg-white text-blue-600">
            <CardHeader>
              <CardTitle className="text-2xl font-bold text-center">Swap USDC Now</CardTitle>
            </CardHeader>
            <CardContent>
              <form onSubmit={handleSubmit} className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="amount" className="text-blue-600">USDC Amount</Label>
                  <Input
                    id="amount"
                    placeholder="Enter USDC amount"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    required
                    className="border-blue-200 focus:border-blue-400"
                  />
                  <BalanceDisplay label="Balance on Near" balance={balances.nearBalance} />
                </div>
                
                <div className="space-y-2">
                  <Label htmlFor="ethAddress" className="text-blue-600">Ethereum Recipient Address</Label>
                  <Input
                    id="ethAddress"
                    placeholder="0x..."
                    value={ethAddress}
                    onChange={(e) => setEthAddress(e.target.value)}
                    required
                    className="border-blue-200 focus:border-blue-400"
                  />
                  <BalanceDisplay label="Balance on Eth" balance={balances.ethBalance} />
                </div>

                {error && (
                  <Alert variant="destructive">
                    <AlertDescription>{error}</AlertDescription>
                  </Alert>
                )}

                {(nearTxHash || txHash) && (
                  <Alert>
                    <AlertDescription>
                      {nearTxHash && !txHash && (
                        <>
                          NEAR Transaction sent! 
                          <a 
                            href={`https://explorer.${wallet.networkId}.near.org/transactions/${nearTxHash}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="ml-1 text-blue-600 hover:underline"
                          >
                            {nearTxHash.slice(0, 8)}...{nearTxHash.slice(-6)}
                          </a>
                          <div className="text-sm text-gray-500 mt-1 flex items-center">
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            Processing NEAR transaction...
                          </div>
                        </>
                      )}
                      {txHash && (
                        <>
                          ETH Transaction {isConfirmed ? 'confirmed!' : 'sent!'} 
                          <a 
                            href={`https://sepolia.etherscan.io/tx/${txHash}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="ml-1 text-blue-600 hover:underline"
                          >
                            {txHash.slice(0, 8)}...{txHash.slice(-6)}
                          </a>
                          {!isConfirmed && (
                            <div className="text-sm text-gray-500 mt-1 flex items-center">
                              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                              Waiting for confirmation...
                            </div>
                          )}
                        </>
                      )}
                    </AlertDescription>
                  </Alert>
                )}

                <Button 
                  type="submit" 
                  className="w-full bg-blue-600 text-white hover:bg-blue-700" 
                  disabled={isLoading}
                >
                  {isLoading ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Processing
                    </>
                  ) : (
                    <>
                      Bridge USDC
                      <ArrowLeftRight className="ml-2 h-4 w-4" />
                    </>
                  )}
                </Button>
              </form>
            </CardContent>
          </Card>
        </main>
      </div>
      <footer className="py-6 text-center text-sm opacity-75 mt-auto">
        © 2024 Direct Transfer. All rights reserved. Powered by NEAR and Ethereum.
      </footer>
    </div>
  )
}
