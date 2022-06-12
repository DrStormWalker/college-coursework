#!/bin/sh

if [ -z "$1" ] || [ $(echo "$1" | cut -c 1) = "-" ]; then
  subcmd="run"
else
  subcmd="$1"
  shift
fi

while [ -n "$1" ] && [ "$1" != "--" ]; do
  if [ -z $cargo_args ]; then
    cargo_args="$1"
  else
    cargo_args="$cargo_args$IFS$1"
  fi
  shift
done

root_dir=$(dirname $(realpath $0))

case $subcmd in
  # Compile and run the program
  run)
    SS_LOG_DIR="$root_dir/env/logs" \
    cargo run \
    $cargo_args \
    $@
    ;;
  # Build the program
  build)
    cargo build \
    $([ $release -eq "1" ] && echo "--release") \
    $@
    ;;
  *)
    echo "Sub-command" $subcmd "Not found"
    exit 1
    ;;
esac

