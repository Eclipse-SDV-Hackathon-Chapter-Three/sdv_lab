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
