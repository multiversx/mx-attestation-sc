#ifndef _TYPES_H_
#define _TYPES_H_

#define NULL 0
#define true 1
#define false 0

typedef unsigned char byte;
typedef unsigned int u32;
typedef int i32;
typedef long long i64;
typedef unsigned long long u64;

typedef int bool;
typedef byte ADDRESS[32];
typedef byte HASH[32];
typedef byte SMALL_ARRAY[100];
typedef byte BIG_ARRAY[1024];

#endif
