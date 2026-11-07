The account design's formal model now covers strict mode (formal/, run
only in CI): every membership change waits as a proposal, a member's
reject made in time stops it, readers follow at most one change per
period, and the recovery phrase together with the remaining device is the
way to remove a stolen device that vetoes its own removal. Nothing ships
to users and no code uses it yet.
