#pragma once

#include "types.h"
#include "unpack.hh"
#include <map>
#include <string>

static inline const u32 CalcHash(const char* val) {
    u32 output{0};
    while (*val != '\0') {
        output = *val + output * 0x11;
        val++;
    }
    return output;
}

std::map<u32, std::string> ReadHashes(const char* path) {
    std::map<u32, std::string> result{};
    unique_ifstream stream = OpenFile(path);
    std::string s{};
    while (std::getline(*stream, s, '\n')) {
        u32 hash = CalcHash(s.c_str());
        if (s.find('#') == 0)
            continue;
        result[hash] = std::string{s};
    }
    return result;
}