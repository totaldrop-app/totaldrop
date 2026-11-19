- **Core:** Receiving into a symbolic link writes to the folder it pointed
  to when the transfer began even if the link changes midway, stopping
  prevents any further file from being placed and waits for in-flight
  placements to finish, and the destination check fails if its test file
  cannot be removed (#92).
