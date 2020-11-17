#include "serde.h"
#include "helpers.h"

void _serializeu32(u32 value, byte *result)
{
    int shift = 32;
    int i;

    for (i = 0; i < sizeof(i32); i++)
    {
        shift -= 8;
        result[i] = (byte)((value >> shift) & 0xff);
    }
}

u32 _deserializeu32(const byte *value)
{
    u32 result = 0;
    int shift = 32;
    int i;

    for (i = 0; i < sizeof(i32); i++)
    {
        shift -= 8;
        result |= ((u32)value[i]) << shift;
    }

    return result;
}

void _serializeu64(u64 value, byte *result)
{
    int shift = 64;
    int i;

    for (i = 0; i < sizeof(u64); i++)
    {
        shift -= 8;
        result[i] = (byte)((value >> shift) & 0xff);
    }
}

u64 _deserializeu64(const byte *value)
{
    u64 result = 0;
    int shift = 64;
    int i;

    for (i = 0; i < sizeof(u64); i++)
    {
        shift -= 8;
        result |= ((u64)value[i]) << shift;
    }

    return result;
}

int _serializeUser(const User *user, byte *result)
{
    byte nonceAsBytes[8] = {};
    byte privateInfoLenAsBytes[4] = {};
    int index = 0;

    result[index] = (byte)user->valueState;
    index++;

    _copyRange(result, user->publicInfo, index, 0, sizeof(HASH));
    index += sizeof(HASH);

    _copyRange(result, user->address, index, 0, sizeof(ADDRESS));
    index += sizeof(ADDRESS);

    _copyRange(result, user->attester, index, 0, sizeof(ADDRESS));
    index += sizeof(ADDRESS);

    _serializeu64(user->nonce, nonceAsBytes);
    _copyRange(result, nonceAsBytes, index, 0, sizeof(u64));
    index += sizeof(u64);

    _serializeu32(user->privateInfoLen, privateInfoLenAsBytes);
    _copyRange(result, privateInfoLenAsBytes, index, 0, sizeof(i32));
    index += sizeof(i32);

    _copyRange(result, user->privateInfo, index, 0, user->privateInfoLen);
    index += user->privateInfoLen;

    return index;
}

int _deserializeUser(const byte *data, User *user)
{
    int index = 0;

    user->valueState = (ValueState)data[index];
    index++;

    _copyRange(user->publicInfo, data, 0, index, sizeof(HASH));
    index += sizeof(HASH);

    _copyRange(user->address, data, 0, index, sizeof(ADDRESS));
    index += sizeof(ADDRESS);

    _copyRange(user->attester, data, 0, index, sizeof(ADDRESS));
    index += sizeof(ADDRESS);

    user->nonce = _deserializeu64(&data[index]);
    index += sizeof(u64);

    user->privateInfoLen = _deserializeu32(&data[index]);
    index += sizeof(i32);

    _copyRange(user->privateInfo, data, 0, index, user->privateInfoLen);
    index += user->privateInfoLen;

    return index;
}
