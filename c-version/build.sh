export PATH="~/elrondsdk/llvm/v9-19feb:${PATH}"

clang-9 -cc1 -emit-llvm -v -triple=wasm32-unknown-unknown-wasm c-version.c serde.c storage.c helpers.c

llc -O2 -filetype=obj c-version.ll -o c-version.o
llc -O2 -filetype=obj serde.ll -o serde.o
llc -O2 -filetype=obj storage.ll -o storage.o
llc -O2 -filetype=obj helpers.ll -o helpers.o

wasm-ld --no-entry c-version.o serde.o storage.o helpers.o -o c-version.wasm --strip-all -allow-undefined -export=init -export=registerData -export=savePublicInfo -export=attest -export=addAttestator -export=setRegisterCost -export=removeAttestator -export=claim -export=getUserData -export=getPublicKey

rm c-version.ll serde.ll storage.ll helpers.ll
rm c-version.o serde.o storage.o helpers.o
mv -f c-version.wasm output
