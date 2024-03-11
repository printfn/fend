$ErrorActionPreference = "Stop"

# https://learn.microsoft.com/en-us/windows/win32/menurc/using-rc-the-rc-command-line-

Set-Location "$PSScriptRoot"

& magick convert icon.svg `
	`( "-clone" 0 "-resize" 16x16 `) `
	`( "-clone" 0 "-resize" 32x32 `) `
	`( "-clone" 0 "-resize" 48x48 `) `
	`( "-clone" 0 "-resize" 64x64 `) `
	`( "-clone" 0 "-resize" 96x96 `) `
	`( "-clone" 0 "-resize" 128x128 `) `
	`( "-clone" 0 "-resize" 256x256 `) `
	"-delete" 0 "-alpha" remove "-colors" 256 fend-icon.ico

& rc /v resources.rc
