#include "elrond/context.h"
#include "elrond/util.h"

#include "user.h"
#include "helpers.h"
#include "serde.h"
#include "storage.h"

ERROR_MSG(ERR_WRONG_FEE, "should pay exactly the registration cost");
ERROR_MSG(ERR_KEY_ALREADY_EXISTS, "is not allowed to save under attestator key");
ERROR_MSG(ERR_ALREADY_REGISTERED, "user already registered");
ERROR_MSG(ERR_ALREADY_PROCESSING, "data already in processing for other user");
ERROR_MSG(ERR_NOT_AN_ATTESTATOR, "caller is not an attestator");
ERROR_MSG(ERR_NOT_THE_SELECTED_ATTESTER, "not the selected attester");
ERROR_MSG(ERR_OUTSIDE_GRACE_PERIOD, "outside of grace period");
ERROR_MSG(ERR_NO_USER_UNDER_KEY, "no user registered under key");
ERROR_MSG(ERR_ONLY_USER_CAN_ATTEST, "only user can attest");
ERROR_MSG(ERR_INFO_MISMATCH, "private/public info mismatch");


GENERAL_MSG(MSG_OK, "ok");

// endpoints

// Args:
// bigInt registrationCost
// ADDRESS address
// u64 maxNonceDiff
void init() 
{
    CHECK_NUM_ARGS(3);
    CHECK_NOT_PAYABLE();
    
    bigInt registrationCost = bigIntNew(0);
    ADDRESS address = {};
    byte maxNonceDiffAsBytes[sizeof(u64)] = {};
    u64 maxNonceDiff;

    bigIntGetUnsignedArgument(0, registrationCost);
    getArgument(1, address);
    getArgument(2, maxNonceDiffAsBytes);
    maxNonceDiff = _deserializeu64(maxNonceDiffAsBytes);

    _storeRegistrationCost(registrationCost);
    _storeAttestorState(address, Approved);
    _storeAttestorList(&address, 1);
    _storeMaxNonceDiff(maxNonceDiff);
}

// PAYABLE
// Args:
// HASH obfuscatedData
void registerData()
{
    CHECK_NUM_ARGS(1);

    bigInt payment = bigIntNew(0);
    bigInt registrationCost = bigIntNew(0);
    HASH obfuscatedData = {};
    ValueState state;
    User user = {};
    ADDRESS caller = {};

    bigIntGetCallValue(payment);
    getArgument(0, obfuscatedData);

    _loadRegistrationCost(registrationCost);
    if (bigIntCmp(payment, registrationCost) != 0)
    {
        SIGNAL_ERROR(ERR_WRONG_FEE);
    }

    state = _loadAttestatorState(obfuscatedData);
    if (state != None)
    {
        SIGNAL_ERROR(ERR_KEY_ALREADY_EXISTS);
    }

    _loadUserOrDefault(obfuscatedData, &user);    
    if (user.valueState == Approved)
    {
        SIGNAL_ERROR(ERR_ALREADY_REGISTERED);
    }

    getCaller(caller);
    if (_isZero(user.address, sizeof(ADDRESS)))
    {
        _copy(user.address, caller, sizeof(ADDRESS));
    }
    else if (!_equal(user.address, caller, sizeof(ADDRESS)))
    {
        if (getBlockNonce() - user.nonce < _loadMaxNonceDiff())
        {
            SIGNAL_ERROR(ERR_ALREADY_PROCESSING);
        }

        _copy(user.address, caller, sizeof(ADDRESS));
    }
    if (_isZero(user.attester, sizeof(ADDRESS)))
    {
        // select attestator
    }

    user.nonce = getBlockNonce();
    if (user.valueState != Pending)
    {
        user.valueState = Requested;
    }

    _storeUser(obfuscatedData, &user);

    finish(MSG_OK, MSG_OK_LEN);
}

// Args:
// HASH obfuscated data
// HASH public info
void savePublicInfo()
{
    CHECK_NOT_PAYABLE();
    CHECK_NUM_ARGS(2);

    HASH obfuscatedData = {};
    HASH publicInfo = {};
    ADDRESS caller = {};
    ValueState state;
    User user = {};
    u64 blockNonce;

    getArgument(0, obfuscatedData);
    getArgument(1, publicInfo);

    getCaller(caller);
    state = _loadAttestatorState(caller);
    if (state == None)
    {
        SIGNAL_ERROR(ERR_NOT_AN_ATTESTATOR);
    }

    _loadUserOrDefault(obfuscatedData, &user);
    if (user.valueState == Approved)
    {
        SIGNAL_ERROR(ERR_ALREADY_REGISTERED);
    }

    if (_isZero(user.address, sizeof(ADDRESS)))
    {
        _copy(user.attester, caller, sizeof(ADDRESS));
    }
    else if (!_equal(user.attester, caller, sizeof(ADDRESS)))
    {
        SIGNAL_ERROR(ERR_NOT_THE_SELECTED_ATTESTER);
    }

    blockNonce = getBlockNonce();
    if (blockNonce - user.nonce > _loadMaxNonceDiff())
    {
        SIGNAL_ERROR(ERR_OUTSIDE_GRACE_PERIOD);
    }

    _copy(user.publicInfo, publicInfo, sizeof(HASH));
    user.nonce = blockNonce;
    user.valueState = Pending;

    _storeUser(obfuscatedData, &user);

    finish(MSG_OK, MSG_OK_LEN);
}

// Args:
// HASH obfuscatedData
// byte[] privateInfo
void attest()
{
    CHECK_NOT_PAYABLE();
    CHECK_NUM_ARGS(2);

    HASH obfuscatedData = {};
    BIG_ARRAY privateInfo = {};
    int privateInfoLen;
    HASH privateInfoHash = {};
    User user = {};
    ADDRESS caller = {};

    getArgument(0, obfuscatedData);
    privateInfoLen = getArgument(1, privateInfo);

    if (_storageUserIsEmpty(obfuscatedData))
    {
        SIGNAL_ERROR(ERR_NO_USER_UNDER_KEY);
    }

    _loadUser(obfuscatedData, &user);
    if (user.valueState != Pending)
    {
        SIGNAL_ERROR(ERR_ALREADY_REGISTERED);
    }

    getCaller(caller);
    if (!_equal(user.address, caller, sizeof(ADDRESS)))
    {
        SIGNAL_ERROR(ERR_ONLY_USER_CAN_ATTEST);
    }
    
    keccak256(privateInfo, privateInfoLen, privateInfoHash);
    if (!_equal(privateInfoHash, user.publicInfo, sizeof(HASH)))
    {
        SIGNAL_ERROR(ERR_INFO_MISMATCH);
    }

    if (getBlockNonce() - user.nonce > _loadMaxNonceDiff())
    {
        SIGNAL_ERROR(ERR_OUTSIDE_GRACE_PERIOD);
    }

    _copy(user.privateInfo, privateInfo, privateInfoLen);
    user.privateInfoLen = privateInfoLen;
    user.valueState = Approved;
    _storeUser(obfuscatedData, &user);

    finish(MSG_OK, MSG_OK_LEN);
}

// view functions

// private functions

void _selectAttestator(ADDRESS attestator)
{
    // TO DO
}

