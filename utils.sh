kontxt_tag () {
    kontxt unit list | fzf | awk '{print $1}'
}

kontxt_jump () {
    emacs -nw "$(kontxt unit show --format json "$(kontxt_tag)" | jq -r '[.repo.location.inner.Local.path, .file] | join("/")')"
}
