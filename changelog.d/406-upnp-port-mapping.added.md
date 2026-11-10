- **Linux:** Routers that offer UPnP but not PCP or NAT-PMP are now also
  asked to open a port during a transfer. As with the other two, the port
  is asked for two minutes at a time, renewed while the transfer runs and
  closed when it ends; a router that says it only opens ports permanently
  is not used. Privacy mode still asks the router nothing (#406).
