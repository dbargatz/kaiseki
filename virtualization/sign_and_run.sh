#!/usr/bin/env zsh
codesign -f --entitlement virtualization.entitlements -s - "$1"
exec "$@"