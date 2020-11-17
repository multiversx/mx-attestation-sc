#ifndef _STORAGE_H_
#define _STORAGE_H_

#include "elrond/bigInt.h"
#include "elrond/types.h"

#include "user.h"

void _loadRegistrationCost(bigInt cost);
void _storeRegistrationCost(bigInt cost);
u64 _loadMaxNonceDiff();
void _storeMaxNonceDiff(u64 nonce);
int _loadAttestatorListLen();
void _loadAttestorList(ADDRESS *attestors);
void _storeAttestorList(const ADDRESS *attestors, int len);
ValueState _loadAttestatorState(const ADDRESS attestator);
void _storeAttestorState(const ADDRESS attestator, ValueState state);
bool _storageUserIsEmpty(const HASH obfuscatedData);
void _loadUser(const HASH obfuscatedData, User *user);
void _storeUser(const HASH obfuscatedData, const User *user);

#endif
