import { useMetaMask } from '../hooks/useMetaMask'
import { sepolia } from 'wagmi/chains'

export function ConnectWallet() {
  const { account, connect, disconnect, isConnected, chainId } = useMetaMask()

  const truncate = (addr: string) =>
    `${addr.slice(0, 6)}...${addr.slice(-4)}`

  if (!isConnected) {
    return (
      <button
        onClick={connect}
        className="bg-orange-500 hover:bg-orange-600 text-white font-semibold px-4 py-2 rounded-lg transition-colors"
      >
        Connect MetaMask
      </button>
    )
  }

  return (
    <div className="flex items-center gap-3">
      {chainId === sepolia.id ? (
        <span className="bg-orange-500/20 text-orange-400 border border-orange-500/40 text-xs px-2 py-1 rounded-full">
          Sepolia
        </span>
      ) : (
        <span className="bg-red-500/20 text-red-400 border border-red-500/40 text-xs px-2 py-1 rounded-full">
          Wrong Network
        </span>
      )}
      <span className="text-slate-300 font-mono text-sm">
        {account ? truncate(account) : ''}
      </span>
      <button
        onClick={() => disconnect()}
        className="text-slate-500 hover:text-slate-300 text-xs underline"
      >
        Disconnect
      </button>
    </div>
  )
}
