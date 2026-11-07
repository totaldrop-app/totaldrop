- **All platforms:** The transfer socket asks the system for buffers of up
  to 4 MiB each way, so bursts of packets on a fast local network are less
  likely to be dropped by the receiver and slow the transfer down. How
  much a device grants depends on its system (#341).
