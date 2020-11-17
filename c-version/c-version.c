#include "elrond/context.h"
#include "elrond/util.h"

#include "user.h"
#include "helpers.h"
#include "serde.h"

const ADDRESS ZERO_32_BYTE_ARRAY = { 0 };

// full keys
STORAGE_KEY(REGISTRATION_COST); // -> BigInt
STORAGE_KEY(MAX_NONCE_DIFF); // -> u64
STORAGE_KEY(ATTESTATOR_LIST); // -> ADDRESS[]

// partial keys
STORAGE_KEY(ATTESTATOR); // + ADDRESS -> ValueState
STORAGE_KEY(USER); // + HASH -> User

// endpoints

// Args:
// bigInt registration cost
void init() 
{
    CHECK_NUM_ARGS(1);
    CHECK_NOT_PAYABLE();
    
    /*bigInt registrationCost = bigIntNew(0);
    bigIntGetUnsignedArgument(0, registrationCost);
    bigIntStorageStoreUnsigned(REGISTRATION_COST_KEY, REGISTRATION_COST_KEY_LEN, 
        registrationCost);*/
}

// view functions

// storage

void _constructKey(const byte *prefix, int prefixLen,  const byte *arg, int argLen, byte *key)
{
    _copy(key, prefix, prefixLen);
    _copyRange(key, arg, prefixLen, 0, argLen);
}

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

int _loadAttestatorListLen()
{
    int lenInBytes = storageLoadLength(ATTESTATOR_LIST_KEY, ATTESTATOR_LIST_KEY_LEN);

    return lenInBytes / sizeof(ADDRESS);
}

void _loadAttestorList(ADDRESS *attestors)
{
    storageLoad(ATTESTATOR_LIST_KEY, ATTESTATOR_LIST_KEY_LEN, (byte*)attestors);
}

void _storeAttestorList(const ADDRESS *attestors, int len)
{
    storageStore(ATTESTATOR_LIST_KEY, ATTESTATOR_LIST_KEY_LEN, (const byte*)attestors, len * sizeof(ADDRESS));
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

void _storeAttestorState(const ADDRESS attestator, ValueState state)
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

void _loadUser(const HASH obfuscatedData, User *user)
{
    const int keyLen = USER_KEY_LEN + sizeof(HASH);
    byte key[keyLen] = {};
    byte serialized[sizeof(User)];

    _constructKey(USER_KEY, USER_KEY_LEN, obfuscatedData, sizeof(HASH), key);
    storageLoad(key, keyLen, serialized);
    _deserializeUser(serialized, user);
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
