#!/bin/bash
# Description: Installer for OFS + mtfg-rs

arg1="$1"

if [ "$EUID" -eq 0 ]; then
    echo "ERROR: You can not run this script with sudo!!"
    exit 1
fi

nix_setup=0
echo "install required packages"
if command -v apt; then
    # debian based distro:
    sudo apt install -y curl

    echo "Install OFS AppImage dependencies"
    sudo apt install -y fuse

    if ! command -v nix; then
        nix_setup=1
        sh <(curl -L https://nixos.org/nix/install) --daemon --yes
    fi
fi

if [ -f /etc/profile.d/nix.sh ]; then
    . /etc/profile.d/nix.sh
fi

if [ -f /home/$USER/.nix-profile/etc/profile.d/nix.sh ]; then
    . /home/$USER/.nix-profile/etc/profile.d/nix.sh
fi

if ! command -v nix; then
    echo "This installer require the package manager nix"
    exit 1
fi

if [ ! -f ~/.config/nix/nix.conf ]; then
    mkdir -p ~/.config/nix
    echo "experimental-features = nix-command flakes" >  ~/.config/nix/nix.conf
    sudo systemctl restart nix-daemon.service
fi

OFS_APP_DIR="$HOME/.local/share/OFS/application"
OFS_EXTENSION_DIR="$HOME/.local/share/OFS/OFS3_data/extensions"

ofs_appimage_download_url=$(curl -s -H "Accept: application/vnd.github.v3+json" https://api.github.com/repos/OpenFunscripter/OFS/releases/latest | grep -Eo "https://.*64x.*AppImage")

if ! command -v OpenFunscripter; then
    echo "OFS AppImage Download URL: $ofs_appimage_download_url"
    mkdir -p $OFS_APP_DIR/bin/data
    rm -rf $OFS_APP_DIR/bin/OpenFunscripter
    wget -c "$ofs_appimage_download_url" -O $OFS_APP_DIR/bin/OpenFunscripter
    wget -c https://raw.githubusercontent.com/OpenFunscripter/OFS/master/data/logo64.png -O $OFS_APP_DIR/bin/data/logo64.png
    chmod +x $OFS_APP_DIR/bin/OpenFunscripter
fi

echo ">> Install ofs extension"
mkdir -p "$OFS_EXTENSION_DIR/mtfg-rs"
pushd "$OFS_EXTENSION_DIR/mtfg-rs"

if [ ! -d "$OFS_EXTENSION_DIR/mtfg-rs/mtfg-rs/.git" ]; then
    git clone https://github.com/michael-mueller-git/mtfg-rs.git
fi

pushd $OFS_EXTENSION_DIR/mtfg-rs/mtfg-rs

echo "Update mtfg-rs"
git reset --hard HEAD
git clean -fd
git remote prune origin
git checkout main
git pull

if [ "$arg1" != "--latest" ]; then
    echo "Checkout latest mtfg-rs release"
    git -c advice.detachedHead=false checkout $(git describe --tags `git rev-list --tags --max-count=1`)
else
    echo "Use latest git commit (recommend only for developers!)"
    if git branch -a | grep -q "next" ; then
        echo "Switch to 'next' branch"
        git checkout next
        git pull
    fi
fi

echo "build nix environment"
nix build ".#release"

popd

cp -fv "$OFS_EXTENSION_DIR/mtfg-rs/mtfg-rs/contrib/OpenFunscripter/extension/main.lua" \
    "$OFS_EXTENSION_DIR/mtfg-rs/main.lua"

cp -fv "$OFS_EXTENSION_DIR/mtfg-rs/mtfg-rs/contrib/OpenFunscripter/extension/json.lua" \
    "$OFS_EXTENSION_DIR/mtfg-rs/json.lua"

if [ ! -e ~/.local/bin/OpenFunscripter ]; then
    mkdir -p ~/.local/bin
    ln -s `realpath $OFS_APP_DIR`/bin/OpenFunscripter ~/.local/bin/OpenFunscripter
fi

mkdir -p ~/.local/share/applications

cat >~/.local/share/applications/OpenFunscripter.desktop <<EOL
[Desktop Entry]
Type=Application
Name=OpenFunscripter
Exec=`realpath $OFS_APP_DIR`/bin/OpenFunscripter
Comment=OpenFunscripter
StartupWMClass=OpenFunscripter
Icon=`realpath $OFS_APP_DIR`/bin/data/logo64.png
EOL

echo -e "\n"
echo "INFO: Installation Finished"

if [ "$arg1" = "--latest" ]; then
    echo "WARNING: you have install the latest application code"
fi

if [ $nix_setup -gt 0 ]; then
    echo "You need to restart you computer to get mtfg-rs working"
fi
