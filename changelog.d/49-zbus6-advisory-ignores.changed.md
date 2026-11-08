- **CI:** cargo-deny and OSV-Scanner no longer ignore any advisory, since
  the zbus 5 crates left the dependency tree, and the check that their
  lists agree now reads every form cargo-deny accepts (#49).
