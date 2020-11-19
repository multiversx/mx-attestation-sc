export PATH="~/elrondsdk/llvm/v9-19feb:${PATH}"

clang-9 -cc1 -emit-llvm -triple=wasm32-unknown-unknown-wasm c-version.c serde.c storage.c helpers.c

llc -O0 -filetype=obj c-version.ll -o c-version.o
llc -O0 -filetype=obj serde.ll -o serde.o
llc -O0 -filetype=obj storage.ll -o storage.o
llc -O0 -filetype=obj helpers.ll -o helpers.o

wasm-ld --no-entry c-version.o serde.o storage.o helpers.o -o c-version.wasm --strip-all -allow-undefined -export=init
