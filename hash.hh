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

template<typename ... Args>
std::string string_format( const std::string& format, Args ... args )
{
    int size_s = std::snprintf( nullptr, 0, format.c_str(), args ... ) + 1; // Extra space for '\0'
    if( size_s <= 0 ){ throw std::runtime_error( "Error during formatting." ); }
    auto size = static_cast<size_t>( size_s );
    std::unique_ptr<char[]> buf( new char[ size ] );
    std::snprintf( buf.get(), size, format.c_str(), args ... );
    return std::string( buf.get(), buf.get() + size - 1 ); // We don't want the '\0' inside
}