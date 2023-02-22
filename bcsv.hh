#pragma once

#include "unpack.hh"
#include "hash.hh"
#include <vector>
#include <string>

namespace BCSV {
    struct Header {
        u32 entrycount;
        u32 fieldcount;
        u32 entrydataoff;
        u32 entrysize;
    };
    template <bool Swap = false>
    Header ReadHeader(unique_ifstream& data) {
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
        u32 mask = 0xFFFFFFFF;
        u16 dataoff;
        u8 shift;
        u8 type;
    };
    template <bool Swap = false>
    std::vector<Field> ReadFields(unique_ifstream& data, Header& head) {
        std::vector<Field> result{head.fieldcount};
        for (int i = 0; i < head.fieldcount; i++) {
            Field* f = &result[i];
            std::tuple<u32, u32, u16, u8, u8> tup = unpack<Swap, u32, u32, u16, u8, u8>(data);
            unpacktuple(tup, &f->hash, &f->mask, &f->dataoff, &f->shift, &f->type);
        }
        return result;
    }
    static inline const size_t GetDTSize(Field& f) {
        switch (f.type) {
            case DataType::LONG:
            case DataType::LONG_2:
            case DataType::FLOAT:
            case DataType::STRING_OFF: { return 4; }
            case DataType::STRING: { return 32; }
            case DataType::SHORT: { return 2; }
            case DataType::CHAR: { return 1; }
            default: { return 0; }
        }
    }
    static inline const char* GetDTFmt(Field& f) {
        switch (f.type) {
            case DataType::LONG:
            case DataType::SHORT:
            case DataType::LONG_2: { return "%d"; }
            case DataType::FLOAT: { return "%f"; }
            case DataType::CHAR: { return "%d"; }
            case DataType::STRING:
            case DataType::STRING_OFF: { return "%s"; }
            default: { return nullptr; }
        }
    }
    template <bool Swap = true>
    std::string ReadStringOff(unique_ifstream& data, Header& head, u32 row, Field& field) {
        std::streampos stringoff = head.entrydataoff + head.entrycount*head.entrysize;
        std::streampos posoff = head.entrydataoff + row * head.entrysize + field.dataoff;
        std::streampos old = data->tellg();
        data->seekg(posoff, std::ios::beg);
        u32 size{};
        data->read((char*)&size, sizeof(u32));
        if (Swap)
            SwapVal(size);
        data->seekg(stringoff + std::streampos{size}, std::ios::beg);
        std::string result{};
        std::getline(*data, result, '\000');
        data->seekg(old);
        return result;
    }
    template <typename T, bool Swap = true>
    T ReadType(unique_ifstream& data, Header& head, u32 row, Field& field) {
        std::streampos posoff = head.entrydataoff + row * head.entrysize + field.dataoff;
        std::streampos old = data->tellg();
        data->seekg(posoff, std::ios::beg);
        T result{};
        data->read((char*)&result, sizeof(T));
        if (Swap && sizeof(T) > 1)
            SwapVal(result);
        data->seekg(old, std::ios::beg);
        return result;
    }
}