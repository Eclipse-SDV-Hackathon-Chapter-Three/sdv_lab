#!/bin/bash
set -euo pipefail

# ---- general build tooling ----
sudo apt update
sudo apt install -y build-essential libssl-dev pkg-config

# ---- install rust toolchain ----
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# ---- fetch clang-12 (focal) packages into a temp dir ----
mkdir -p ~/tmp/clang12 && cd ~/tmp/clang12
base=https://archive.ubuntu.com/ubuntu/pool/universe/l/llvm-toolchain-12

wget ${base}/clang-12_12.0.1-19ubuntu3_amd64.deb
wget ${base}/libclang-12-dev_12.0.1-19ubuntu3_amd64.deb
wget ${base}/libclang1-12_12.0.1-19ubuntu3_amd64.deb
wget ${base}/libclang-cpp12_12.0.1-19ubuntu3_amd64.deb
wget ${base}/llvm-12_12.0.1-19ubuntu3_amd64.deb
wget ${base}/llvm-12-dev_12.0.1-19ubuntu3_amd64.deb
wget ${base}/llvm-12-tools_12.0.1-19ubuntu3_amd64.deb
wget ${base}/lld-12_12.0.1-19ubuntu3_amd64.deb
wget ${base}/libllvm12_12.0.1-19ubuntu3_amd64.deb
wget ${base}/libclang-common-12-dev_12.0.1-19ubuntu3_amd64.deb  # contains stddef.h etc.

# ---- install into isolated prefix ----
sudo mkdir -p /opt/llvm-12
for f in *.deb; do sudo dpkg-deb -x "$f" /opt/llvm-12; done

# ---- handy entrypoint symlinks (safe) ----
sudo ln -sf /opt/llvm-12/usr/bin/clang-12 /usr/local/bin/clang-12
sudo ln -sf /opt/llvm-12/usr/bin/clang++-12 /usr/local/bin/clang++-12
sudo ln -sf /opt/llvm-12/usr/bin/llvm-config-12 /usr/local/bin/llvm-config-12

# ---- environment for bindgen/autocxx + general use ----
# (append once; idempotent check keeps ~/.bashrc tidy)
grep -q '# clang-12 for bindgen/autocxx' ~/.bashrc || cat >> ~/.bashrc <<'EOF'
# clang-12 for bindgen/autocxx
export PATH=/opt/llvm-12/usr/bin:$PATH
export LLVM_CONFIG_PATH=/opt/llvm-12/usr/bin/llvm-config-12
export CLANG_PATH=/opt/llvm-12/usr/bin/clang-12
export LIBCLANG_PATH=/opt/llvm-12/usr/lib/llvm-12/lib
export LIBCLANG_STATIC_PATH=/opt/llvm-12/usr/lib/llvm-12/lib
export RD=/opt/llvm-12/usr/lib/llvm-12/lib/clang/12.0.1
export BINDGEN_EXTRA_CLANG_ARGS="--target=x86_64-unknown-linux-gnu -resource-dir=$RD -isystem $RD/include -msse2 -mssse3 -msse4.1"
export AUTOCXX_EXTRA_CLANG_ARGS="$BINDGEN_EXTRA_CLANG_ARGS"
# If you still hit loader issues, uncomment the next line:
# export LD_LIBRARY_PATH=/opt/llvm-12/usr/lib/x86_64-linux-gnu:/opt/llvm-12/usr/lib/llvm-12/lib:$LD_LIBRARY_PATH
EOF

echo "Done. Open a new shell or run: source ~/.bashrc"
