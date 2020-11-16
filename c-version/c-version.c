#include "elrond/context.h"
#include "elrond/util.h"

#include "user.h"
#include "helpers.h"
#include "serde.h"

const ADDRESS ZERO_32_BYTE_ARRAY = { 0 };

STORAGE_KEY(REGISTRATION_COST);

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
