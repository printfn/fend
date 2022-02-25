# This can be added to a .zshrc file to make fend nicer to use
# on the command-line. It makes sure that globbing is disabled, which
# means that e.g. `fend 2 * 3` will work as expected (instead of zsh
# trying to expand `*` into a list of files).
#
# You can try out how this works by running `source noglob.sh`, or you
# can simply copy-paste this into your ~/.zshrc file.
#
# Note that this only works in zsh, not in bash or any other shell.
#

alias fend="nocorrect noglob command fend"
