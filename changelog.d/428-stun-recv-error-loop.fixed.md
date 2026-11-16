- **Core:** Finding the device's public address via STUN no longer stops
  waiting when an unreachable STUN address triggers a socket reset on
  Windows, so a working IPv4 or IPv6 STUN reply on the other family is
  still kept (#428).
