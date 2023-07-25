# clndir

CLI tool that deletes old files from a directory.

It defaults to a directory that is specified in the Downloads ENV variable. You can pass a directory on the command line with the --dir option.<br>
It defaults to deleting files that were updated more than 180 days ago. You can override with the --age option.<br>
It defaults to showing you the list of files it is intending to delete and will prompt the user to confirm. You can override this with the --nowarn flag.<br>
You can specify patterns to not delete. These are specified with the --skip parameter.<br>

Written as an exercise to learn Rust 