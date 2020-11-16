#ifndef _SERDE_H_
#define _SERDE_H_

#include "elrond/types.h"
#include "user.h"

void _serializeu32(u32 value, byte *result);
u32 _deserializeu32(const byte *value);
void _serializeu64(u64 value, byte *result);
u64 _deserializeu64(const byte *value);
int _serializeUser(const User *user, byte *result);
int _deserializeUser(const byte *data, User *user);

#endif
