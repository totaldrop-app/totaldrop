The account design's formal model now covers strict mode (formal/, run
only in CI): a single admin signature changes membership only after the
wait, any member's reject stops it, and the recovery phrase is the way to
remove a stolen device that vetoes its own removal. Nothing ships to users
and no code uses it yet.
