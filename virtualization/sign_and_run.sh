#!/usr/bin/env zsh
codesign -f --entitlement $(PWD)/../virtualization/virtualization.entitlements -s - "$1"
exec "$@"