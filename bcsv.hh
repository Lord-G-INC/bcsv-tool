#pragma once

#include "unpack.hh"
#include "types.h"

namespace BCSV {
    struct Header {
        u32 entrycount;
        u32 fieldcount;
        u32 entrydataoff;
        u32 entrysize;
    };
    template <bool Swap = false>
    Header ReadHeader(unsigned char* data) {
        Header result{};
        std::tuple<u32, u32, u32, u32> tup = unpack<Swap, u32, u32, u32, u32>(data);
        unpacktuple(tup, &result.entrycount, &result.fieldcount, &result.entrydataoff, &result.entrysize);
        return result;
    }
    namespace DataType {
        const u8 LONG = 0x0;
        const u8 STRING = 0x1;
        const u8 FLOAT = 0x2;
        const u8 LONG_2 = 0x3;
        const u8 SHORT = 0x4;
        const u8 CHAR = 0x5;
        const u8 STRING_OFF = 0x6;
    }
    struct Field {
        u32 hash;
        u32 mask;
        u16 dataoff;
        u8 shift;
        u8 type;
    };
    template <bool Swap = false>
    Field* ReadFields(unsigned char* data, Header& head) {
        Field* result = new Field[head.fieldcount];
        for (int i = 0; i < head.fieldcount; i++) {
            Field* f = &result[i];
            std::tuple<u32, u32, u16, u8, u8> tup = unpack<Swap, u32, u32, u16, u8, u8>(data);
            unpacktuple(tup, &f->hash, &f->mask, &f->dataoff, &f->shift, &f->type);
            data += sizeof(Field);
        }
        return result;
    }
}