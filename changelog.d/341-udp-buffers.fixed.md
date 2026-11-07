- **All platforms:** Receiving over a fast local network no longer stalls
  at a fraction of the sending speed when the receiver drops packets that
  arrive in bursts: the transfer socket asks the system for 4 MiB buffers
  each way (#341).
