#!/usr/bin/env sh

codesign --sign - --entitlements hv.entitlements.xml --deep --force target/debug/hello-hv