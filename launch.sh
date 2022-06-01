#!/bin/sh

if [ -z "$1" ] || [ $(echo "$1" | cut -c 1) = "-" ]; then
  subcmd="run"
else
  subcmd="$1"
fi

while [ -n "$1" ] && [ "$1" != "--" ]; do
  case $1 in
    "--release")
      release="1"
      shift
      ;;
    *)
      shift
      ;;
  esac
done

root_dir=$(dirname $(realpath $0))

case $subcmd in
  # Compile and run the program
  run)
    SS_LOG_DIR="$root_dir/env/logs" \
    cargo run \
    $([ $release -eq "1" ] && echo "--release") \
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

