import { useState, useRef, useEffect } from 'react'
import { api } from '../hooks/useGrantBot'

interface Props {
  repoId: string
}

const SEPOLIA_ETHERSCAN = 'https://sepolia.etherscan.io/tx'

type Phase =
  | 'idle'
  | 'pending'
  | 'approved'
  | 'executing'
  | 'paid'
  | 'rejected'
  | 'failed'

function phaseLabel(phase: Phase): string {
  switch (phase) {
    case 'idle': return ''
    case 'pending': return '⏳ Pending...'
    case 'approved': return '🤖 Venice thinking...'
    case 'executing': return '💸 Executing payment...'
    case 'paid': return '✅ Paid!'
    case 'rejected': return '❌ Rejected'
    case 'failed': return '⚠️ Failed'
  }
}

function phaseColor(phase: Phase): string {
  switch (phase) {
    case 'paid': return 'text-green-400'
    case 'rejected': return 'text-red-400'
    case 'failed': return 'text-orange-400'
    default: return 'text-yellow-400'
  }
}

export function TestPanel({ repoId }: Props) {
  const { triggerTestPR, pollPRStatus } = api
  const [form, setForm] = useState({
    pr_number: 1,
    pr_title: 'Fix critical bug in auth flow',
    contributor: 'contributor1',
  })
  const [phase, setPhase] = useState<Phase>('idle')
  const [txHash, setTxHash] = useState<string | null>(null)
  const [veniceReason, setVeniceReason] = useState('')
  const [amount, setAmount] = useState<number | null>(null)
  const [loading, setLoading] = useState(false)

  // Track mount state and pending timer to cancel on unmount
  const mountedRef = useRef(true)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    mountedRef.current = true
    return () => {
      mountedRef.current = false
      if (timerRef.current) clearTimeout(timerRef.current)
    }
  }, [])

  const handleSimulate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (timerRef.current) clearTimeout(timerRef.current)

    setLoading(true)
    setPhase('pending')
    setTxHash(null)
    setVeniceReason('')
    setAmount(null)

    try {
      const { pr_event_id } = await triggerTestPR({ repo_id: repoId, ...form })

      const poll = async () => {
        if (!mountedRef.current) return

        try {
          const result = await pollPRStatus(pr_event_id)
          if (!mountedRef.current) return

          const s = result.status

          if (s === 'paid') {
            setPhase('paid')
            if (result.payment) {
              setTxHash(result.payment.tx_hash)
              setVeniceReason(result.payment.venice_reason)
              setAmount(result.payment.amount)
            }
            setLoading(false)
          } else if (s === 'rejected') {
            setPhase('rejected')
            if (result.payment) setVeniceReason(result.payment.venice_reason)
            setLoading(false)
          } else if (s === 'failed') {
            setPhase('failed')
            setLoading(false)
          } else {
            // still pending — show "venice thinking" and keep polling
            setPhase('approved')
            timerRef.current = setTimeout(poll, 2000)
          }
        } catch {
          if (mountedRef.current) {
            setPhase('failed')
            setLoading(false)
          }
        }
      }

      timerRef.current = setTimeout(poll, 2000)
    } catch {
      if (mountedRef.current) {
        setPhase('failed')
        setLoading(false)
      }
    }
  }

  return (
    <div className="bg-slate-800/60 border border-slate-700 rounded-xl p-5">
      <h2 className="text-white font-semibold text-lg mb-4">
        Test Panel
        <span className="ml-2 text-xs text-slate-400 font-normal">Simulate a merged PR</span>
      </h2>

      <form onSubmit={handleSimulate} className="space-y-3 mb-4">
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-slate-400 block mb-1">PR Number</label>
            <input
              type="number"
              className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-purple-500"
              value={form.pr_number}
              onChange={(e) => setForm((f) => ({ ...f, pr_number: Number(e.target.value) }))}
              required
            />
          </div>
          <div>
            <label className="text-xs text-slate-400 block mb-1">Contributor GitHub</label>
            <input
              className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-purple-500"
              placeholder="github_username"
              value={form.contributor}
              onChange={(e) => setForm((f) => ({ ...f, contributor: e.target.value }))}
              required
            />
          </div>
        </div>
        <input
          className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-purple-500"
          placeholder="PR Title"
          value={form.pr_title}
          onChange={(e) => setForm((f) => ({ ...f, pr_title: e.target.value }))}
          required
        />
        <button
          type="submit"
          disabled={loading}
          className="w-full bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white font-semibold py-2 rounded-lg transition-colors text-sm"
        >
          {loading ? 'Simulating...' : '🚀 Simulate Merged PR'}
        </button>
      </form>

      {phase !== 'idle' && (
        <div className="bg-slate-900/80 rounded-lg p-4 space-y-2">
          <div className={`text-sm font-semibold ${phaseColor(phase)}`}>
            {phaseLabel(phase)}
          </div>

          {veniceReason && (
            <div className="text-xs text-slate-400">
              <span className="text-slate-500">🤖 Venice: </span>
              <span className="italic">"{veniceReason}"</span>
            </div>
          )}

          {phase === 'paid' && amount != null && (
            <div className="text-xs text-green-400">
              Paid {amount} USDC
              {txHash && (
                <a
                  href={`${SEPOLIA_ETHERSCAN}/${txHash}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="ml-2 text-blue-400 hover:underline font-mono"
                >
                  {txHash.slice(0, 14)}... ↗
                </a>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
