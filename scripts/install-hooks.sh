#!/bin/sh
# install-hooks.sh: run scripts/check-public.sh before every commit and every push of this
# checkout.
#
#   scripts/install-hooks.sh
#
# Writes .git/hooks/pre-commit and .git/hooks/pre-push (in a linked worktree: the hooks
# directory git names). A hook that already exists and was not written by this script is left
# alone and named. The hooks read ORGSPEC_DENYLIST from the environment of the git command.

set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
hooks=$(git rev-parse --git-path hooks)
mkdir -p "$hooks"

mark="# written by scripts/install-hooks.sh"

for name in pre-commit pre-push; do
  hook=$hooks/$name
  if [ -e "$hook" ] && ! grep -q "^$mark\$" "$hook"; then
    echo "install-hooks: $hook exists and is not ours; left alone" >&2
    continue
  fi
  {
    echo '#!/bin/sh'
    echo "$mark"
    echo 'exec "$(git rev-parse --show-toplevel)/scripts/check-public.sh"'
  } > "$hook"
  chmod +x "$hook"
  echo "install-hooks: wrote $hook"
done
