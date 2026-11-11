- **Linux:** When a router answers both PCP and NAT-PMP, the second port
  it opens is now closed as soon as the router answers, and a port the
  two share is released under both protocols when the transfer ends,
  instead of staying open, unused, for up to two minutes (#425).
