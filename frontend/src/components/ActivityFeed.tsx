import { useState, useEffect } from 'react'
import { api, type Payment } from '../hooks/useGrantBot'

interface Props {
  repoId: string
}

const SEPOLIA_ETHERSCAN = 'https://sepolia.etherscan.io/tx'

function statusColor(status: string) {
  switch (status) {
    case 'paid': return 'bg-green-900/40 border-green-700/40'
    case 'rejected': return 'bg-red-900/40 border-red-700/40'
    case 'pending': return 'bg-yellow-900/40 border-yellow-700/40'
    default: return 'bg-slate-800/60 border-slate-700/40'
  }
}

function statusIcon(status: string) {
  switch (status) {
    case 'paid': return '💸'
    case 'rejected': return '🚫'
    case 'pending': return '⏳'
    case 'failed': return '⚠️'
    default: return '🔀'
  }
}

export function ActivityFeed({ repoId }: Props) {
  const [payments, setPayments] = useState<Payment[]>([])

  useEffect(() => {
    let cancelled = false

    const poll = async () => {
      try {
        const data = await api.getPayments(repoId)
        if (!cancelled) setPayments(data)
      } catch {
        // ignore network errors during polling
      }
    }

    poll()
    const id = setInterval(poll, 4000)
    return () => {
      cancelled = true
      clearInterval(id)
    }
  }, [repoId])

  if (payments.length === 0) {
    return (
      <div className="bg-slate-800/60 border border-slate-700 rounded-xl p-5">
        <h2 className="text-white font-semibold text-lg mb-3">Activity Feed</h2>
        <p className="text-slate-500 text-sm text-center py-8">
          No activity yet. Merge a PR to see payments appear here.
        </p>
      </div>
    )
  }

  return (
    <div className="bg-slate-800/60 border border-slate-700 rounded-xl p-5">
      <h2 className="text-white font-semibold text-lg mb-3">
        Activity Feed
        <span className="ml-2 w-2 h-2 bg-green-400 rounded-full inline-block animate-pulse" />
      </h2>
      <div className="space-y-2 max-h-72 overflow-y-auto pr-1">
        {payments.map((p) => (
          <div
            key={p.id}
            className={`border rounded-lg px-3 py-3 text-sm ${statusColor(p.status)}`}
          >
            <div className="flex items-start justify-between gap-2">
              <div>
                <span className="mr-2">{statusIcon(p.status)}</span>
                <span className="text-slate-300 font-medium">PR #{p.pr_number}</span>
                <span className="text-slate-400"> — "{p.pr_title}" by </span>
                <span className="text-purple-400">@{p.contributor}</span>
              </div>
              <span className="text-xs text-slate-500 whitespace-nowrap">
                {new Date(p.created_at ?? '').toLocaleTimeString()}
              </span>
            </div>
            <div className="mt-1 ml-6 text-xs text-slate-400">
              <span className="mr-1">🤖 Venice:</span>
              <span className="italic">"{p.venice_reason}"</span>
            </div>
            {p.status === 'paid' && (
              <div className="mt-1 ml-6 text-xs text-green-400">
                ✅ Paid {p.amount} USDC
                {p.tx_hash && (
                  <a
                    href={`${SEPOLIA_ETHERSCAN}/${p.tx_hash}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="ml-2 text-blue-400 hover:underline font-mono"
                  >
                    {p.tx_hash.slice(0, 10)}...
                  </a>
                )}
              </div>
            )}
            {p.status === 'rejected' && (
              <div className="mt-1 ml-6 text-xs text-red-400">❌ Rejected</div>
            )}
            {p.status === 'failed' && (
              <div className="mt-1 ml-6 text-xs text-orange-400">⚠️ Failed — check logs</div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
