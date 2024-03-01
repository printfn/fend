#!/usr/bin/env bash
set -exuo pipefail
docker pull archlinux:latest
docker run -ti archlinux bash -euxc \
	"pacman -Syyu git curl go sudo base-devel --noconfirm
	useradd -m -G wheel -s /bin/bash user
	echo \"%wheel ALL=(ALL) NOPASSWD: ALL\" | sudo EDITOR=\"tee -a\" visudo
	sudo -u user bash -euxc \"
		cd
		git clone https://aur.archlinux.org/yay.git
		cd yay
		makepkg -si --noconfirm
		yay --noconfirm -Syu aur/fend
		fend \\\"1 kg to lbs\\\"
	\""
