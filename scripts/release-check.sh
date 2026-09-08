#!/usr/bin/env bash

set -euo pipefail

if [[ -n "$(git status --porcelain --untracked-files=all)" ]]; then
    echo "release check failed: the working tree must be clean" >&2
    exit 1
fi

qhull_package_id=$(cargo pkgid --package qhull)
qhull_sys_package_id=$(cargo pkgid --package qhull-sys)
qhull_version=${qhull_package_id##*#}
qhull_version=${qhull_version##*@}
qhull_sys_version=${qhull_sys_package_id##*#}
qhull_sys_version=${qhull_sys_version##*@}

if [[ "$qhull_version" != "$qhull_sys_version" ]]; then
    echo "release check failed: qhull ($qhull_version) and qhull-sys ($qhull_sys_version) must have the same version" >&2
    exit 1
fi

expected_tag="v$qhull_version"
if ! git tag --points-at HEAD --format='%(refname:short)' | grep --fixed-strings --line-regexp --quiet "$expected_tag"; then
    echo "release check failed: HEAD must have the $expected_tag tag" >&2
    exit 1
fi

echo "release check passed for $expected_tag"
