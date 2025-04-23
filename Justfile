# Just is a task runner, like Make but without the build system / dependency tracking part.
# docs: https://github.com/casey/just
#
# The `-ci` variants are ran in CI, they do command grouping on GitHub Actions, set consistent env vars etc.,
# but they require bash.
#
# The non`-ci` variants can be run locally without having bash installed.

set dotenv-load

default: precommit prepush

precommit: code-quality
prepush: clippy test

ci: precommit prepush docs

clippy-all:
    cargo clippy --workspace --all-targets --all-features --target-dir target/clippy-all-features -- -D warnings

clippy:
    cargo clippy --workspace --all-targets --target-dir target/clippy -- -D warnings

test *args:
    cargo nextest run {{args}} < /dev/null

test-ci *args:
    #!/usr/bin/env -S bash -euo pipefail
    source .envrc
    echo -e "\033[1;33m🏃 Running all but doc-tests with nextest...\033[0m"
    cmd_group "cargo nextest run --features slow-tests {{args}} < /dev/null"

    echo -e "\033[1;36m📚 Running documentation tests...\033[0m"
    cmd_group "cargo test --features slow-tests --doc {{args}}"

doc-tests *args:
    cargo test --doc {{args}}

doc-tests-ci *args:
    #!/usr/bin/env -S bash -euo pipefail
    source .envrc
    echo -e "\033[1;36m📚 Running documentation tests...\033[0m"
    cmd_group "cargo test --doc {{args}}"

code-quality:
    cargo fmt --check --all

ship:
    #!/usr/bin/env -S bash -euo pipefail
    # Refuse to run if not on master branch or not up to date with origin/master
    branch="$(git rev-parse --abbrev-ref HEAD)"
    if [[ "$branch" != "master" ]]; then
    echo -e "\033[1;31m❌ Refusing to run: not on 'master' branch (current: $branch)\033[0m"
    exit 1
    fi
    git fetch origin master
    local_rev="$(git rev-parse HEAD)"
    remote_rev="$(git rev-parse origin/master)"
    if [[ "$local_rev" != "$remote_rev" ]]; then
    echo -e "\033[1;31m❌ Refusing to run: local master branch is not up to date with origin/master\033[0m"
    echo -e "Local HEAD:  $local_rev"
    echo -e "Origin HEAD: $remote_rev"
    echo -e "Please pull/rebase to update."
    exit 1
    fi
    release-plz update
    git add .
    git commit -m "Upgrades"
    git push
    just publish

publish:
    git_token := $(gh auth token 2>/dev/null) || echo $PUBLISH_GITHUB_TOKEN
    release-plz release --backend github --git-token $git_token

docsrs *args:
    #!/usr/bin/env -S bash -eux
    source .envrc
    export RUSTDOCFLAGS="--cfg docsrs"
    cargo +nightly doc {{args}}

docs:
    cargo doc --workspace --all-features --no-deps --document-private-items --keep-going

lockfile:
    cargo update --workspace --locked
