#include "storage.h"

#include "elrond/util.h"
#include "helpers.h"
#include "serde.h"

// full keys
STORAGE_KEY(REGISTRATION_COST); // -> BigInt
STORAGE_KEY(MAX_NONCE_DIFF); // -> u64
STORAGE_KEY(TOTAL_ATTESTATORS); // -> u64

// partial keys
STORAGE_KEY(LIST_ATTESTATOR) // + index -> ADDRESS
STORAGE_KEY(ATTESTATOR); // + ADDRESS -> ValueState
STORAGE_KEY(USER); // + HASH -> User

const ADDRESS ZERO_32_BYTE_ARRAY = { 0 };

void _loadRegistrationCost(bigInt cost)
{
    bigIntStorageLoadUnsigned(REGISTRATION_COST_KEY, REGISTRATION_COST_KEY_LEN, cost);
}

void _storeRegistrationCost(bigInt cost)
{
    bigIntStorageStoreUnsigned(REGISTRATION_COST_KEY, REGISTRATION_COST_KEY_LEN, cost);
}

u64 _loadMaxNonceDiff()
{
    byte serialized[sizeof(u64)] = {};

    storageLoad(MAX_NONCE_DIFF_KEY, MAX_NONCE_DIFF_KEY_LEN, serialized);

    return _deserializeu64(serialized); 
}

void _storeMaxNonceDiff(u64 nonce)
{
    byte serialized[sizeof(u64)] = {};

    _serializeu64(nonce, serialized);
    storageStore(MAX_NONCE_DIFF_KEY, MAX_NONCE_DIFF_KEY_LEN, serialized, sizeof(u64));
}

u64 _loadAttestorListLen()
{
    return smallIntStorageLoadUnsigned(TOTAL_ATTESTATORS_KEY, TOTAL_ATTESTATORS_KEY_LEN);
}

void _storeAttestorListLen(u64 len)
{
    smallIntStorageStoreUnsigned(TOTAL_ATTESTATORS_KEY, TOTAL_ATTESTATORS_KEY_LEN, len);
}

void _loadAttestatorAt(u64 index, ADDRESS attestator)
{
    const int keyLen = LIST_ATTESTATOR_KEY_LEN + sizeof(i64);
    byte key[keyLen] = {};
    byte serializedIndex[sizeof(u64)] = {};

    _serializeu64(index, serializedIndex);
    _constructKey(LIST_ATTESTATOR_KEY, LIST_ATTESTATOR_KEY_LEN, 
        serializedIndex, sizeof(u64), key);
    
    storageLoad(key, keyLen, attestator);
}

void _storeNewAttestator(const ADDRESS attestator)
{
    const int keyLen = LIST_ATTESTATOR_KEY_LEN + sizeof(i64);
    byte key[keyLen] = {};
    u64 index = _loadAttestorListLen();
    byte serializedIndex[sizeof(u64)] = {};

    _serializeu64(index, serializedIndex);
    _constructKey(LIST_ATTESTATOR_KEY, LIST_ATTESTATOR_KEY_LEN, 
        serializedIndex, sizeof(u64), key);

    storageStore(key, keyLen, attestator, sizeof(ADDRESS));
    _storeAttestorListLen(index + 1);
}

// deleting just moves the last element and overwrites the "deleted one"
// this is done to maintain a contiguous list
void _deleteAttestatorAt(u64 index)
{
    const int keyLen = LIST_ATTESTATOR_KEY_LEN + sizeof(i64);
    byte deletedKey[keyLen] = {};
    byte serializedDeletedIndex[sizeof(u64)] = {};
    u64 lastIndex = _loadAttestorListLen() - 1;
    ADDRESS lastAddress = {};

    _loadAttestatorAt(lastIndex, lastAddress);

    _serializeu64(index, serializedDeletedIndex);
    _constructKey(LIST_ATTESTATOR_KEY, LIST_ATTESTATOR_KEY_LEN, 
        serializedDeletedIndex, sizeof(u64), deletedKey);
    storageStore(deletedKey, keyLen, lastAddress, sizeof(ADDRESS));
    _storeAttestorListLen(lastIndex); // i.e. prev len - 1
}

ValueState _loadAttestatorState(const ADDRESS attestator)
{
    const int keyLen = ATTESTATOR_KEY_LEN + sizeof(ADDRESS);
    byte key[keyLen] = {};
    byte result;

    _constructKey(ATTESTATOR_KEY, ATTESTATOR_KEY_LEN, attestator, sizeof(ADDRESS), key);
    storageLoad(key, keyLen, &result);

    return (ValueState)result;
}

void _storeAttestatorState(const ADDRESS attestator, ValueState state)
{
    const int keyLen = ATTESTATOR_KEY_LEN + sizeof(ADDRESS);
    byte key[keyLen] = {};

    _constructKey(ATTESTATOR_KEY, ATTESTATOR_KEY_LEN, attestator, sizeof(ADDRESS), key);
    storageStore(key, keyLen, (byte*)&state, sizeof(byte));
}

bool _storageUserIsEmpty(const HASH obfuscatedData)
{
    const int keyLen = USER_KEY_LEN + sizeof(HASH);
    byte key[keyLen] = {};
    int storageLen;

    _constructKey(USER_KEY, USER_KEY_LEN, obfuscatedData, sizeof(HASH), key);
    storageLen = storageLoadLength(key, keyLen);

    return storageLen == 0 ? true : false;
}

int _loadUserRaw(const HASH obfuscatedData, byte *user)
{
    const int keyLen = USER_KEY_LEN + sizeof(HASH);
    byte key[keyLen] = {};

    _constructKey(USER_KEY, USER_KEY_LEN, obfuscatedData, sizeof(HASH), key);
    
    return storageLoad(key, keyLen, user);
}

void _loadUser(const HASH obfuscatedData, User *user)
{
    byte serialized[sizeof(User)];

    _loadUserRaw(obfuscatedData, serialized);
    _deserializeUser(serialized, user);
}

void _loadUserOrDefault(const HASH obfuscatedData, User *user)
{
    if (!_storageUserIsEmpty(obfuscatedData))
    {
        _loadUser(obfuscatedData, user);
    }
    else
    {
        user->valueState = None;
        _copy(user->publicInfo, ZERO_32_BYTE_ARRAY, sizeof(HASH));
        _copy(user->address, ZERO_32_BYTE_ARRAY, sizeof(ADDRESS));
        _copy(user->attester, ZERO_32_BYTE_ARRAY, sizeof(ADDRESS));
        user->nonce = getBlockNonce();
        user->privateInfoLen = 0;
        // no need to initialize user->privateInfo
    }
}

void _storeUser(const HASH obfuscatedData, const User *user)
{
    const int keyLen = USER_KEY_LEN + sizeof(HASH);
    byte key[keyLen] = {};
    byte serialized[sizeof(User)];
    int serializedLen;

    _constructKey(USER_KEY, USER_KEY_LEN, obfuscatedData, sizeof(HASH), key);
    serializedLen = _serializeUser(user, serialized);
    storageStore(key, keyLen, serialized, serializedLen);
}
