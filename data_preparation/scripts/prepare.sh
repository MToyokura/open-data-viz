#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DATA_PREPARATION_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd -- "$DATA_PREPARATION_DIR/.." && pwd)"

if [[ -f "$REPO_ROOT/.env" ]]; then
  if [[ ${ESTAT_APP_ID+x} ]]; then
    estat_app_id_was_set=1
    estat_app_id_from_environment=$ESTAT_APP_ID
  else
    estat_app_id_was_set=0
  fi

  set -a
  # shellcheck disable=SC1090
  source "$REPO_ROOT/.env"
  set +a

  if [[ $estat_app_id_was_set == 1 ]]; then
    export ESTAT_APP_ID=$estat_app_id_from_environment
  fi
fi

cd "$DATA_PREPARATION_DIR"
exec cargo "$@"
