The Flatpak CI job builds against the runtime and SDK already pinned in its
container image instead of updating them in place from Flathub (#582);
nothing in the shipped packages changes.
