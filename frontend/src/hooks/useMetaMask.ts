import { useAccount, useConnect, useDisconnect, useSwitchChain } from 'wagmi'
import { sepolia } from 'wagmi/chains'
import { useEffect } from 'react'

export function useMetaMask() {
  const { address, isConnected, chainId } = useAccount()
  const { connect, connectors } = useConnect()
  const { disconnect } = useDisconnect()
  const { switchChain } = useSwitchChain()

  useEffect(() => {
    if (isConnected && chainId !== sepolia.id) {
      switchChain({ chainId: sepolia.id })
    }
  }, [isConnected, chainId, switchChain])

  const connectWallet = () => {
    const injected = connectors.find((c) => c.id === 'injected')
    if (injected) connect({ connector: injected })
  }

  return {
    account: address,
    connect: connectWallet,
    disconnect,
    isConnected,
    chainId,
  }
}
