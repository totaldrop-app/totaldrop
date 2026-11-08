- **CI:** Every pull request and every push to main is scanned for
  secrets with gitleaks, and fails when a signing key or a
  `key.properties` file is committed (#378).
