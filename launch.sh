#!/bin/sh

if [ -z "$1" ] || [ $1 = "--" ]; then
  subcmd=""
else
  subcmd="$1"
fi

while [ -n "$1" ] && [ "$1" != "--" ]; do
  shift 1
done

root_dir=$(dirname $(realpath $0))
echo $root_dir

case $subcmd in
  run)
    SS_LOG_DIR="$root_dir/env/logs" \
    cargo run $@
    ;;
  build)
    cargo build $@
    ;;
  *)
    echo "Sub-command" $subcmd "Not found"
    exit 1
    ;;
esac

