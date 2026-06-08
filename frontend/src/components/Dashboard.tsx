import { useState } from 'react'
import { ConnectWallet } from './ConnectWallet'
import { RegisterRepo } from './RegisterRepo'
import { GrantDelegation } from './GrantDelegation'
import { AddContributor } from './AddContributor'
import { ActivityFeed } from './ActivityFeed'
import { TestPanel } from './TestPanel'
import { StatsBar } from './StatsBar'
import { type Repo } from '../hooks/useGrantBot'

interface Props {
  ownerAddress: string
}

export function Dashboard({ ownerAddress }: Props) {
  const [repo, setRepo] = useState<Repo | null>(null)
  const [delegationHex, setDelegationHex] = useState<string | null>(null)

  const handleRepoRegistered = (r: Repo) => {
    setRepo(r)
    setDelegationHex(r.delegation_hex)
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* Header */}
      <header className="border-b border-slate-800 px-6 py-4">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-2xl">🤖</span>
            <div>
              <h1 className="text-white font-bold text-xl leading-none">GrantBot</h1>
              <p className="text-slate-500 text-xs">AI-powered USDC grants for OSS contributors</p>
            </div>
          </div>
          <ConnectWallet />
        </div>
      </header>

      <div className="max-w-7xl mx-auto px-6 py-6 space-y-6">
        {/* Stats Bar */}
        {repo && <StatsBar repoId={repo.id} />}

        <div className="grid grid-cols-5 gap-6">
          {/* Left Panel: Setup */}
          <div className="col-span-2 space-y-4">
            <RegisterRepo
              ownerAddress={ownerAddress}
              onRegistered={handleRepoRegistered}
            />
            {repo && (
              <>
                <GrantDelegation
                  repoId={repo.id}
                  dailyCap={repo.daily_cap}
                  delegationHex={delegationHex}
                  onGranted={() => setDelegationHex('granted')}
                />
                <AddContributor repoId={repo.id} />
              </>
            )}
          </div>

          {/* Right Panel: Activity + Test */}
          <div className="col-span-3 space-y-4">
            {repo ? (
              <>
                <ActivityFeed repoId={repo.id} />
                <TestPanel repoId={repo.id} />
              </>
            ) : (
              <div className="bg-slate-800/40 border border-slate-700/50 border-dashed rounded-xl p-12 text-center">
                <div className="text-4xl mb-4">👈</div>
                <p className="text-slate-400">Register a repository to start monitoring PRs</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
