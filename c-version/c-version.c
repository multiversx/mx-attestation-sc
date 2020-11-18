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
ERROR_MSG(ERR_FORBIDDEN, "only the contract owner may call this function");
ERROR_MSG(ERR_DOES_NOT_EXIST, "attestator does not exist");
ERROR_MSG(ERR_CANNOT_DELETE_LAST, "cannot delete last attestator");
ERROR_MSG(ERR_USER_DATA_NOT_ATTESTED, "userData not yet attested");

GENERAL_MSG(MSG_OK, "ok");
GENERAL_MSG(MSG_CLAIM, "attestation claim");

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
    _storeAttestatorState(address, Approved);
    _storeNewAttestator(address);
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

// Owner-only
// Args:
// ADDRESS attestator
void addAttestator()
{
    CHECK_NOT_PAYABLE();
    CHECK_NUM_ARGS(1);

    ADDRESS attestator = {};
    ADDRESS owner = {};
    ADDRESS caller = {};
    ValueState state;

    getArgument(0, attestator);

    getOwnerAddress(owner);
    getCaller(caller);
    if (!_equal(caller, owner, sizeof(ADDRESS)))
    {
        SIGNAL_ERROR(ERR_FORBIDDEN);
    }

    state = _loadAttestatorState(attestator);
    if (state != None)
    {
        SIGNAL_ERROR(ERR_KEY_ALREADY_EXISTS);
    }
    if (!_storageUserIsEmpty(attestator))
    {
        SIGNAL_ERROR(ERR_ALREADY_REGISTERED);
    }

    _storeAttestatorState(attestator, Approved);
    _storeNewAttestator(attestator);

    finish(MSG_OK, MSG_OK_LEN);
}

// Owner-only
// Args:
// bigInt registrationCost
void setRegisterCost()
{
    CHECK_NOT_PAYABLE();
    CHECK_NUM_ARGS(1);

    bigInt registrationCost = bigIntNew(0);

    bigIntGetUnsignedArgument(0, registrationCost);

    if (!_isCallerOwner())
    {
        SIGNAL_ERROR(ERR_FORBIDDEN);
    }

    _storeRegistrationCost(registrationCost);

    finish(MSG_OK, MSG_OK_LEN);
}

// Owner-only
// Args:
// ADDRESS attestator
void removeAttestator()
{
    CHECK_NOT_PAYABLE();
    CHECK_NUM_ARGS(1);

    ADDRESS attestator = {};
    ADDRESS tempAttestator = {};
    ValueState state;
    u64 i;
    u64 len;

    getArgument(0, attestator);

    if (!_isCallerOwner())
    {
        SIGNAL_ERROR(ERR_FORBIDDEN);
    }

    state = _loadAttestatorState(attestator);
    if (state == None)
    {
        SIGNAL_ERROR(ERR_DOES_NOT_EXIST);
    }

    len = _loadAttestorListLen();
    if (len == 1)
    {
        SIGNAL_ERROR(ERR_CANNOT_DELETE_LAST);
    }

    for (i = 0; i < len; i++)
    {
        _loadAttestatorAt(i, tempAttestator);
        if (_equal(tempAttestator, attestator, sizeof(ADDRESS)))
        {
            _storeAttestatorState(attestator, None);
            _deleteAttestatorAt(i);
            break;
        }
    }

    finish(MSG_OK, MSG_OK_LEN);
}

// Owner-only
void claim()
{
    CHECK_NOT_PAYABLE();
    CHECK_NUM_ARGS(0);

    ADDRESS owner = {};
    ADDRESS scAddress = {};
    byte balance[32] = {};

    if (!_isCallerOwner())
    {
        SIGNAL_ERROR(ERR_FORBIDDEN);
    }

    getOwnerAddress(owner);
    getSCAddress(scAddress);
    getExternalBalance(scAddress, balance);
    transferValue(owner, balance, MSG_CLAIM, MSG_CLAIM_LEN);

    finish(MSG_OK, MSG_OK_LEN);
}

// view functions

// Args:
// HASH obfuscatedData
// returns: serialized User
void getUserData()
{
    CHECK_NOT_PAYABLE();
    CHECK_NUM_ARGS(1);

    HASH obfuscatedData = {};
    byte rawUser[sizeof(User)] = {};
    int len;

    getArgument(0, obfuscatedData);

    if (_storageUserIsEmpty(obfuscatedData))
    {
        SIGNAL_ERROR(ERR_NO_USER_UNDER_KEY);
    }

    len = _loadUserRaw(obfuscatedData, rawUser);

    finish(rawUser, len);
}

// Args:
// HASH obfuscatedData
// returns: ADDRESS
void getPublicKey()
{
    HASH obfuscatedData = {};
    User user = {};

    getArgument(0, obfuscatedData);

    if (_storageUserIsEmpty(obfuscatedData))
    {
        SIGNAL_ERROR(ERR_NO_USER_UNDER_KEY);
    }

    _loadUser(obfuscatedData, &user);
    if (user.valueState == Approved)
    {
        SIGNAL_ERROR(ERR_USER_DATA_NOT_ATTESTED);
    }

    finish(user.address, sizeof(ADDRESS));
}

// private functions

// TO DO: Randomize the choice
// Currently keeping it like this to mirror the Rust version
void _selectAttestator(ADDRESS attestator)
{
    _loadAttestatorAt(_loadAttestorListLen() - 1, attestator);
}
