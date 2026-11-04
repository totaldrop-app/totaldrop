- **Linux:** The desktop app's embedded Qt resources no longer carry the
  build time of the files cxx-qt generates, so they are the same on every
  build of the same source. One difference remains, the order in which
  cxx-qt links the Qt libraries (#252, #256).
