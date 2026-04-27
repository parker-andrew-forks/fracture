rm -rf launcher
rm -rf fracture
echo "Building launcher with cargo"
cd launcher-crate
cargo build --release
cp target/release/gsc-launcher ../launcher
cd ..
cd ..
echo "Building project with cargo"
cargo build --release --features flatpak 
cp target/release/fracture flatpak/fracture
echo "Building flatpak"
cd flatpak
flatpak-builder --force-clean --user --install-deps-from=flathub --repo=repo --install builddir systems.fracture.launcher.yml
echo "Running flatpak"
flatpak run systems.fracture.launcher
