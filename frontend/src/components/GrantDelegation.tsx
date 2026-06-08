import { useState } from 'react'
import { api } from '../hooks/useGrantBot'

interface Props {
  repoId: string
  dailyCap: number
  delegationHex: string | null
  onGranted: () => void
}

export function GrantDelegation({ repoId, dailyCap, delegationHex, onGranted }: Props) {
  const { storeDelegation } = api
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const requestDelegation = async () => {
    setLoading(true)
    setError('')
    try {
      // In a real integration with @metamask/delegation-toolkit, you would:
      // 1. Use createCaveat() to create a spend cap caveat
      // 2. Use grantPermission() to get an ERC-7715 delegation
      // 3. Encode the delegation as hex
      //
      // For demo/testnet purposes, we prompt the user to paste a delegation hex.
      // Production would use: const delegation = await wallet.grantPermission({ ... })
      const hex = window.prompt(
        `Paste your ERC-7715 delegation hex for ${dailyCap} USDC spending cap (or use MetaMask delegation toolkit):`
      )
      if (!hex) {
        setLoading(false)
        return
      }
      await storeDelegation(repoId, hex)
      onGranted()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to store delegation')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="bg-slate-800/60 border border-slate-700 rounded-xl p-5">
      <h2 className="text-white font-semibold text-lg mb-2">Grant Spending Delegation</h2>
      <p className="text-slate-400 text-sm mb-4">
        Authorize GrantBot to spend up to <strong className="text-white">{dailyCap} USDC</strong> on Sepolia via ERC-7715.
      </p>

      <div className="flex items-center gap-3 mb-4">
        <span className={`text-sm font-medium ${delegationHex ? 'text-green-400' : 'text-red-400'}`}>
          {delegationHex ? '✅ Delegation Granted' : '❌ Not Granted'}
        </span>
      </div>

      {error && <p className="text-red-400 text-xs mb-3">{error}</p>}

      <button
        onClick={requestDelegation}
        disabled={loading}
        className="w-full bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white font-semibold py-2 rounded-lg transition-colors text-sm"
      >
        {loading ? 'Processing...' : delegationHex ? 'Update Delegation' : 'Grant Delegation'}
      </button>

      <p className="text-slate-500 text-xs mt-2">
        Uses MetaMask ERC-7715 spending permissions → 1Shot permissionless relayer executes on Sepolia
      </p>
    </div>
  )
}
