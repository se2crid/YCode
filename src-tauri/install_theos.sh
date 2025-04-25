#!/usr/bin/env bash

# Error codes + association:
# 1 - Running as root
# 2 - Unsupported platform
# 3 - Dependency issue
# 4 - Unsupported shell
# 5 - Setting $THEOS failed
# 6 - Theos clone failed
# 7 - Toolchain install failed
# 8 - SDK install failed
# 9 - Checkra1n '/opt' setup failed
# 10 - WSL1 fakeroot->fakeroot-tcp failed
# 11 - Enabling Linux binary compat on FreeBSD failed

set -e

# Pretty print
special() {
	printf "\e[0;34m==> \e[1;34mTheos Installer:\e[m %s\n" "$1"
}

update() {
	printf "\n\e[0;36m==> \e[1;36m%s\e[m\n" "$1"
}

common() {
	printf "\n\e[0;37m==> \e[1;37m%s\e[m\n" "$1"
}

error() {
	printf "\e[0;31m==> \e[1;31m%s\e[m\n" "$1"
}


# Root is no bueno
if [[ $EUID -eq 0 ]]; then
	error "Theos should NOT be installed with or run as root (su/sudo)!"
	error "  - Please re-run the installer as a non-root user."
	exit 1
fi


# Common vars
PLATFORM=$(uname)
CSHELL="${SHELL##*/}"
SHELL_ENV="unknown"
if [[ $CSHELL == sh || $CSHELL == bash || $CSHELL == dash ]]; then
	# Bash prioritizes bashrc > bash_profile > profile
	if [[ -f $HOME/.bashrc ]]; then
		SHELL_ENV="$HOME/.bashrc"
	elif [[ -f $HOME/.bash_profile ]]; then
		SHELL_ENV="$HOME/.bash_profile"
	else
		SHELL_ENV="$HOME/.profile"
	fi
elif [[ $CSHELL == zsh ]]; then
	# Zsh prioritizes zshenv > zprofile > zshrc
	if [[ -f $HOME/.zshenv ]]; then
		SHELL_ENV="$HOME/.zshenv"
	elif [[ -f $HOME/.zprofile ]]; then
		SHELL_ENV="$HOME/.zprofile"
	else
		SHELL_ENV="$HOME/.zshrc"
	fi
# TODO
# elif [[ $CSHELL == csh ]]; then
# 	SHELL_ENV="$HOME/.cshrc"
fi

run_with_sudo() {
	if [[ $(uname -r | sed -n 's/.*\( *Microsoft *\).*/\L\1/ip') == microsoft ]]; then
        echo "$SUDO_PASSWORD" | sudo -S "$@"
    else
        pkexec "$@"
	fi
}

set_theos() {
	# Check for $THEOS env var
	update "Checking for \$THEOS environment variable..."
	if ! [[ -z $THEOS ]]; then
		update "\$THEOS is already set to '$THEOS'. Nothing to do here."
	else
		update "\$THEOS has not been set. Setting now..."

		if [[ $SHELL_ENV == unknown ]]; then
			error "Current shell ($CSHELL) is unsupported by this installer. Please set the THEOS environment variable to '~/theos' manually before proceeding."
			exit 4
		fi
        echo "export THEOS=~/theos" >> "$SHELL_ENV"
        export THEOS=~/theos
	fi
}

get_theos() {
	# Get Theos
	update "Checking for Theos install..."
	if [[ -d $THEOS && $(ls -A "$THEOS") ]]; then
		update "Theos appears to already be installed. Checking for updates..."
		$THEOS/bin/update-theos
	else
		update "Theos does not appear to be installed. Cloning now..."
		git clone --recursive https://github.com/theos/theos.git $THEOS \
			&& update "Git clone of Theos was successful!" \
			|| (error "Theos git clone command seems to have encountered an error. Please see the log above."; exit 6)
	fi
}

get_sdks() {
	# Get patched sdks
	update "Checking for patched SDKs..."
	if [[ -d $THEOS/sdks/ && $(ls -A "$THEOS/sdks/" | grep sdk) ]]; then
		update "SDKs appear to already be installed."
	else
		update "SDKs do not appear to be installed. Installing now..."
		# Grab latest for provided platforms
		urls=$(curl https://api.github.com/repos/theos/sdks/releases/latest | grep download_url | sed 's/.*: "\(.*\)"/\1/')
		ios_url=$(echo "$urls" | grep 'iPhoneOS' | sort -V | tail -n1)
		tvos_url=$(echo "$urls" | grep 'AppleTVOS' | sort -V | tail -n1)
		curl -L "$ios_url" | tar -xJv -C "$THEOS/sdks"
		curl -L "$tvos_url" | tar -xJv -C "$THEOS/sdks"

		if [[ -d $THEOS/sdks/ && $(ls -A "$THEOS/sdks/" | grep sdk) ]]; then
			update "SDKs successfully installed!"
		else
			error "Something appears to have gone wrong. Please try again."
			exit 8
		fi
	fi
}

linux() {
	# Determine distro
	DISTRO="unknown"
	if [[ -x $(command -v apt) ]]; then
		DISTRO="debian"
	elif [[ -x $(command -v pacman) ]]; then
		DISTRO="arch"
	elif [[ -x $(command -v dnf) ]]; then
		DISTRO="redhat"
	elif [[ -x $(command -v zypper) ]]; then
		DISTRO="suse"
	fi

	# Check for pkexec (not installed by default on some distros)
	if ! [[ -x $(command -v pkexec) ]] && ! [[ $(uname -r | sed -n 's/.*\( *Microsoft *\).*/\L\1/ip') == microsoft ]]; then
		error "Please install 'pkexec' before proceeding with the installation."
		exit 3
	fi

	# Dependencies
	update "Preparing to install dependencies. Please enter your password if prompted:"
	case $DISTRO in
		debian)
			run_with_sudo apt update || true
			run_with_sudo apt install -y build-essential fakeroot rsync curl perl zip git libxml2 \
				&& update "Dependencies have been successfully installed!" \
				|| (error "Dependency install command seems to have encountered an error. Your password may have been incorrect."; exit 3)
			;;
		arch)
			run_with_sudo pacman -Syu || true
			run_with_sudo pacman -S --needed --noconfirm base-devel libbsd fakeroot openssl rsync curl perl zip git libxml2 \
				&& update "Dependencies have been successfully installed!" \
				|| (error "Dependency install command seems to have encountered an error. Your password may have been incorrect."; exit 3)
			;;
		redhat)
			run_with_sudo dnf --refresh || true
			run_with_sudo dnf group install -y "C Development Tools and Libraries" \
				&& update "Dependencies have been successfully installed!" \
				|| (error "Dependency install command seems to have encountered an error. Your password may have been incorrect."; exit 3)
			run_with_sudo dnf install -y fakeroot lzma libbsd rsync curl perl zip git libxml2 \
				&& update "Other dependencies have been successfully installed!" \
				|| (error "Other Dependency install command seems to have encountered an error. Your password may have been incorrect."; exit 3)
			;;
		suse)
			run_with_sudo zypper refresh || true
			run_with_sudo zypper install -y -t pattern devel_basis \
				&& update "Dependencies have been successfully installed!" \
				|| (error "Dependency install command seems to have encountered an error. Your password may have been incorrect."; exit 3)
			run_with_sudo zypper install -y fakeroot libbsd0 rsync curl perl zip git libxml2 \
				&& update "Other dependencies have been successfully installed!" \
				|| (error "Other Dependency install command seems to have encountered an error. Your password may have been incorrect."; exit 3)
			;;
		*)
			error "The dependencies for your distro are unknown to this installer. Note that they will need to be determined before Theos can be installed and/or function properly."
			common "On Debian-based distros, the necessary dependencies are: build-essential fakeroot rsync curl perl git libxml2 and libtinfo5 (non-swift toolchain) or libz3-dev (swift toolchain)."
			common "Additional dependencies may also be required depending on what your distro provides."
			;;
	esac

	# Check for WSL
	update "Checking for WSL..."
	if [[ $(uname -r | sed -n 's/.*\( *Microsoft *\).*/\L\1/ip') == microsoft ]]; then
		VERSION=$(uname -r | sed 's/.*\([[:digit:]]\)[[:space:]]*/\1/')
		if [[ $VERSION -eq 1 ]]; then
			update "WSL1! Need to fix fakeroot..."
			run_with_sudo update-alternatives --set fakeroot /usr/bin/fakeroot-tcp \
				&& update "fakeroot fixed!" \
				|| (error "fakeroot fix seems to have encountered an error. Please see the log above."; exit 10)
		else
			update "WSL2! Nothing to do here."
		fi
	else
		update "Seems you're not using WSL. Moving on..."
	fi

	set_theos
	get_theos

	# Get a toolchain
	update "Checking for iOS toolchain..."
	if [[ -d $THEOS/toolchain/linux/iphone/ && $(ls -A "$THEOS/toolchain/linux/iphone") ]]; then
		update "A toolchain appears to already be installed."
	else
		update "A toolchain does not appear to be installed."
        case $DISTRO in
            debian)
                run_with_sudo apt install -y libz3-dev zstd
                ;;
            arch)
                run_with_sudo pacman -S --needed --noconfirm libedit z3 zstd
                # libz3-dev equivalent is z3 and we need to create lib version queried
                LATEST_LIBZ3="$(ls -v /usr/lib/ | grep libz3 | tail -n 1)"
                run_with_sudo ln -sf /usr/lib/$LATEST_LIBZ3 /usr/lib/libz3.so.4
                # toolchain looks for a specific libedit
                LATEST_LIBEDIT="$(ls -v /usr/lib/ | grep libedit | tail -n 1)"
                run_with_sudo ln -sf /usr/lib/$LATEST_LIBEDIT /usr/lib/libedit.so.2
                ;;
            redhat)
                run_with_sudo dnf install -y z3-libs zstd
                # libz3-dev equivalent is z3-libs and ...
                LATEST_LIBZ3="$(ls -v /usr/lib64/ | grep libz3 | tail -n 1)"
                run_with_sudo ln -sf /usr/lib64/$LATEST_LIBZ3 /usr/lib64/libz3.so.4
                # toolchain looks for a specific libedit
                LATEST_LIBEDIT="$(ls -v /usr/lib64/ | grep libedit | tail -n 1)"
                run_with_sudo ln -sf /usr/lib64/$LATEST_LIBEDIT /usr/lib64/libedit.so.2
                ;;
            suse)
                run_with_sudo zypper install -y $(zypper search libz3 | tail -n 1 | cut -d "|" -f2) zstd
                # libz3-dev equivalent is libz3-* and ...
                LATEST_LIBZ3="$(ls -v /usr/lib64/ | grep libz3 | tail -n 1)"
                run_with_sudo ln -sf /usr/lib64/$LATEST_LIBZ3 /usr/lib64/libz3.so.4
                # toolchain looks for a specific libedit
                LATEST_LIBEDIT="$(ls -v /usr/lib64/ | grep libedit | tail -n 1)"
                run_with_sudo ln -sf /usr/lib64/$LATEST_LIBEDIT /usr/lib64/libedit.so.2
                ;;
        esac
        mkdir -p $THEOS/toolchain/linux/iphone $THEOS/toolchain/swift
        # If not ubuntu, send a warning
        if [[ $DISTRO != debian ]]; then
            common "Theos toolchain for Swift is only supported on Ubuntu. You may need to install the toolchain manually."
        fi
        # Check if the system is Linux Mint
        if [ -f /etc/upstream-release/lsb-release ]; then
            # Source the lsb-release file
            . /etc/upstream-release/lsb-release
        elif command -v lsb_release &> /dev/null; then
            # Get the release number
            DISTRIB_RELEASE=$(lsb_release -rs)
        else
            # Print a warning and use a default value
            common "Warning: Could not determine Ubuntu version. Using default release number."
            DISTRIB_RELEASE="20.04"
        fi

		# If its 24.04, use 22.04
		if [[ $DISTRIB_RELEASE == 24.04 ]]; then
			DISTRIB_RELEASE="22.04"
		fi

        # Print the release number
        update "Downloading toolchain ubuntu$DISTRIB_RELEASE"

        curl -sL https://github.com/kabiroberai/swift-toolchain-linux/releases/download/v2.3.0/swift-5.8-ubuntu$DISTRIB_RELEASE.tar.xz | tar -xJf - -C $THEOS/toolchain
        ln -s $THEOS/toolchain/linux/iphone $THEOS/toolchain/swift

		# Confirm that toolchain is usable
		if [[ -x $THEOS/toolchain/linux/iphone/bin/clang ]]; then
			update "Successfully installed the toolchain!"
		else
			error "Something appears to have gone wrong -- the toolchain is not accessible. Please try again."
			exit 7
		fi
	fi

	get_sdks
}


if [[ ${PLATFORM,,} == linux ]]; then
	linux
else
	error "'$PLATFORM' is currently unsupported by YCode. Currently, only linux is supported."
	exit 2
fi
special "Theos has been successfully installed! Please restart YCode."