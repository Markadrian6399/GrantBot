import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { wagmiConfig } from './wagmiConfig'
import { useAccount } from 'wagmi'
import { Dashboard } from './components/Dashboard'
import { ConnectWallet } from './components/ConnectWallet'

const queryClient = new QueryClient()

function AppContent() {
  const { address, isConnected } = useAccount()

  if (!isConnected || !address) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950 flex items-center justify-center">
        <div className="text-center max-w-md px-6">
          <div className="text-6xl mb-6">🤖</div>
          <h1 className="text-4xl font-bold text-white mb-3">GrantBot</h1>
          <p className="text-slate-400 text-lg mb-8">
            AI-powered autonomous payments for open-source contributors
          </p>
          <p className="text-slate-500 text-sm mb-6 leading-relaxed">
            GrantBot monitors your GitHub repo for merged PRs and automatically
            pays contributors in USDC — no human approval needed.
          </p>
          <div className="flex flex-col items-center gap-3">
            <ConnectWallet />
            <p className="text-slate-600 text-xs">
              Connect MetaMask to get started · Sepolia testnet
            </p>
          </div>
          <div className="mt-10 grid grid-cols-3 gap-4 text-center text-xs text-slate-500">
            <div className="bg-slate-800/40 rounded-lg p-3">
              <div className="text-slate-300 font-semibold mb-1">GitHub</div>
              <div>Monitors merged PRs via webhook</div>
            </div>
            <div className="bg-slate-800/40 rounded-lg p-3">
              <div className="text-slate-300 font-semibold mb-1">Venice AI</div>
              <div>Evaluates & approves payments</div>
            </div>
            <div className="bg-slate-800/40 rounded-lg p-3">
              <div className="text-slate-300 font-semibold mb-1">1Shot</div>
              <div>Executes USDC on Sepolia</div>
            </div>
          </div>
        </div>
      </div>
    )
  }

  return <Dashboard ownerAddress={address} />
}

function App() {
  return (
    <WagmiProvider config={wagmiConfig}>
      <QueryClientProvider client={queryClient}>
        <AppContent />
      </QueryClientProvider>
    </WagmiProvider>
  )
}

export default App
