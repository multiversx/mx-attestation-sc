#ifndef _USER_H_
#define _USER_H_

#include "elrond/types.h"

typedef enum
{
    None,
    Requested,
    Pending,
    Approved,
} ValueState;

typedef struct
{
    ValueState valueState;
    HASH publicInfo;
    ADDRESS address;
    ADDRESS attester;
    u64 nonce;
    u32 privateInfoLen;
    BIG_ARRAY privateInfo;
} User;

#endif
