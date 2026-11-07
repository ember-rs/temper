# Everything a toolchain downloads stays inside the repo, so nothing here
# touches (or depends on) the user's global ~/.rustup, ~/.kani, or ~/.quint.
env = {
  "RUSTUP_HOME": "${HERMIT_ENV}/.hermit/rustup",
  "KANI_HOME": "${HERMIT_ENV}/.hermit/kani",
  "QUINT_HOME": "${HERMIT_ENV}/.hermit/quint",
  "PATH": "${HERMIT_ENV}/node_modules/.bin:${PATH}",
}

github-token-auth {
}
