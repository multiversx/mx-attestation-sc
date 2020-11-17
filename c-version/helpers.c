#include "helpers.h"

#include "elrond/util.h"

void _copy(byte *dest, const byte *src, int len)
{
    int i;
    for (i = 0; i < len; i++)
    {
        dest[i] = src[i];
    }
}

void _copyRange(byte *dest, const byte *src, int destStart, int srcStart, int len)
{
    int i;
    for (int i = 0; i < len; i++)
    {
        dest[destStart + i] = src[srcStart + i];
    }
}

bool _equal(const byte *op1, const byte *op2, int len)
{
    int i;
    for (i = 0; i < len; i++)
    {
        if (op1[i] != op2[i])
        {
            return false;
        }
    }

    return true;
}

void _constructKey(const byte *prefix, int prefixLen,  const byte *arg, int argLen, byte *key)
{
    _copy(key, prefix, prefixLen);
    _copyRange(key, arg, prefixLen, 0, argLen);
}

bool _isZero(const byte *data, int len)
{
    int i;
    for (i = 0; i < len; i++)
    {
        if (data[i] != 0)
        {
            return false;
        }
    }

    return true;
}

bool _isCallerOwner()
{
    ADDRESS caller = {};
    ADDRESS owner = {};

    getCaller(caller);
    getOwnerAddress(owner);

    return _equal(caller, owner, sizeof(ADDRESS));
}
