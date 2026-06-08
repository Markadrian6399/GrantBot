import { useState, useEffect } from 'react'
import { api, type Stats } from '../hooks/useGrantBot'

interface Props {
  repoId: string
}

function StatCard({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="bg-slate-800/60 border border-slate-700 rounded-xl p-4 text-center">
      <div className="text-2xl font-bold text-white">{value}</div>
      <div className="text-xs text-slate-400 mt-1">{label}</div>
    </div>
  )
}

export function StatsBar({ repoId }: Props) {
  const [stats, setStats] = useState<Stats | null>(null)

  useEffect(() => {
    let cancelled = false

    const poll = async () => {
      try {
        const data = await api.getStats(repoId)
        if (!cancelled) setStats(data)
      } catch {
        // ignore network errors during polling
      }
    }

    poll()
    const id = setInterval(poll, 10000)
    return () => {
      cancelled = true
      clearInterval(id)
    }
  }, [repoId])

  if (!stats) return null

  return (
    <div className="grid grid-cols-4 gap-3">
      <StatCard label="Total Paid (USDC)" value={stats.total_paid.toFixed(2)} />
      <StatCard label="PRs Processed" value={stats.pr_count} />
      <StatCard label="Contributors" value={stats.contributor_count} />
      <StatCard label="Today's Spend" value={`${stats.today_spend.toFixed(2)} USDC`} />
    </div>
  )
}
