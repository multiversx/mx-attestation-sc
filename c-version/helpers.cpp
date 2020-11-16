#include "helpers.h"

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
