#ifndef _STORAGE_H_
#define _STORAGE_H_

#include "elrond/bigInt.h"
#include "elrond/types.h"

#include "user.h"

void _loadRegistrationCost(bigInt cost);
void _storeRegistrationCost(bigInt cost);
u64 _loadMaxNonceDiff();
void _storeMaxNonceDiff(u64 nonce);
u64 _loadAttestorListLen();
void _storeAttestorListLen(u64 len);
void _loadAttestatorAt(u64 index, ADDRESS attestator);
void _storeNewAttestator(const ADDRESS attestator);
void _deleteAttestatorAt(u64 index);
ValueState _loadAttestatorState(const ADDRESS attestator);
void _storeAttestatorState(const ADDRESS attestator, ValueState state);
bool _storageUserIsEmpty(const HASH obfuscatedData);
int _loadUserRaw(const HASH obfuscatedData, byte *user);
void _loadUser(const HASH obfuscatedData, User *user);
void _loadUserOrDefault(const HASH obfuscatedData, User *user);
void _storeUser(const HASH obfuscatedData, const User *user);

#endif
