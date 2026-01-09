# Maintainer: GhostKellz <ghost@ghostkellz.sh>
pkgname=nvproton
pkgver=1.0.0
pkgrel=1
pkgdesc="NVIDIA-optimized Proton Game Launcher with FFI to Zig libraries"
arch=('x86_64')
url="https://github.com/ghostkellz/nvproton"
license=('MIT')
depends=('glibc' 'steam')
makedepends=('rust' 'cargo')
optdepends=(
    'proton-nv: NVIDIA-optimized Proton runtime'
    'nvshader: Shader cache pre-warming'
    'nvlatency: Reflex latency control'
    'nvsync: VRR/G-Sync control'
    'lutris: Lutris game detection'
    'heroic-games-launcher: Epic/GOG game detection'
)
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --target x86_64-unknown-linux-gnu
}

package() {
    cd "$pkgname-$pkgver"

    # CLI binary
    install -Dm755 target/x86_64-unknown-linux-gnu/release/nvproton "$pkgdir/usr/bin/nvproton"

    # Default profiles
    install -dm755 "$pkgdir/usr/share/nvproton/profiles"
    if [ -d profiles ]; then
        install -Dm644 profiles/*.yaml "$pkgdir/usr/share/nvproton/profiles/" 2>/dev/null || true
    fi

    # Documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
