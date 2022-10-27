#!/bin/sh

set -e

main() {
	if ! command -v tar >/dev/null; then
		echo "Error: tar is required to install nitrogen" 1>&2
		exit 1
	fi

	ext=".zip"
	nitrogen_install="${nitrogen_INSTALL:-$HOME/.nitrogen}"
	if [ "$OS" = "Windows_NT" ]; then
		target="x86_64-pc-windows-gnu"
	else
		case $(uname -sm) in
		"Darwin x86_64") target="x86_64-apple-darwin" ;;
		"Darwin arm64") target="Darwin_arm64"
			echo "Error: Official nitrogen builds for Darwin arm64 are not available" 1>&2
			exit 1
			;;
		"Linux aarch64")
			echo "Error: Official nitrogen builds for Linux aarch64 are not available" 1>&2
			exit 1
			;;
		"Linux x86_64")
			target="x86_64-unknown-linux-musl"
			ext=".tar.gz"
			nitrogen_install="${nitrogen_INSTALL:-$HOME/.local}"
			;;
		*)
			echo "Error: Official nitrogen builds for Linux arm64 are not available" 1>&2
			exit 1
			;;
		esac
	fi

	# https://github.com/capeprivacy/nitrogen/releases/download/v0.1.0/nitrogen_v0.1.0_x86_64-apple-darwin.zip
	# https://github.com/capeprivacy/nitrogen/releases/download/v0.1.0/nitrogen_v0.1.0_x86_64-unknown-linux-musl.tar.gz
	# https://github.com/capeprivacy/nitrogen/releases/download/v0.1.0/nitrogen_v0.1.0_x86_64-pc-windows-gnu.zip

	if [ $# -eq 0 ]; then
		# get the redirect url for latest to pull the latest version
		redirect_url=$(curl -s -L -I -o /dev/null -w '%{url_effective}' "https://github.com/capeprivacy/nitrogen/releases/latest/download")
		version=${redirect_url##*/}
		nitrogen_uri="https://github.com/capeprivacy/nitrogen/releases/latest/download/nitrogen_${version}_${target}${ext}"
	else
		nitrogen_uri="https://github.com/capeprivacy/nitrogen/releases/download/${1}/cape_${target}${ext}"
	fi

	bin_dir="$nitrogen_install/bin"
	exe="$bin_dir/nitrogen"

	if [ ! -d "$bin_dir" ]; then
		mkdir -p "$bin_dir"
	fi

	echo "$nitrogen_uri"
	curl --fail --location --progress-bar --output "$exe.tar.gz" "$nitrogen_uri"
	tar -C "$bin_dir" -xzf "$exe.tar.gz"
	chmod +x "$exe"
	rm "$exe.tar.gz"

	echo "nitrogen was installed successfully to $exe"
	if command -v nitrogen >/dev/null; then
		echo "Run 'nitrogen --help' to get started"
	else
		case $SHELL in
		/bin/zsh) shell_profile=".zshrc" ;;
		*) shell_profile=".bashrc" ;;
		esac
		echo "Run the following to make nitrogen accessible globally:"
		echo "  sudo cp $bin_dir/nitrogen /usr/local/bin/"
		echo "Run '$exe --help' to get started"
	fi

}
main "$@"
