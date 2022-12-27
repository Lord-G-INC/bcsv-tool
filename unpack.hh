#pragma once

#include <tuple>
#include <stddef.h>
#include <array>
#include <algorithm>
#include <fstream>
#include <iostream>
#include <memory>
#include "types.h"

template <typename T>
    void SwapVal(T& val) {
        union U
        {
            T raw;
            std::array<u8, sizeof(T)> buf;
        } src, dst;
        src.raw = val;
        std::reverse_copy(src.buf.begin(), src.buf.end(), dst.buf.begin());
        val = dst.raw;
    }

template<typename... Types>
constexpr size_t GetTypesSize() {
    std::array<std::size_t, sizeof...(Types)> sizes{sizeof(Types)...};
    size_t result{};
    std::for_each(sizes.begin(), sizes.end(), [&](size_t& s){result += s;});
    return result;
}

template <bool Swap, typename... Args>
std::tuple<Args...> unpack(unsigned char* data);

template <typename... Args>
void unpacktuple(std::tuple<Args...>&, Args*...);

template <class Stream>
struct StreamDel {};

template <>
struct StreamDel<std::ifstream> {
    void operator()(std::ifstream* s) {
        if (s->is_open()) {
            s->close();
        }
        delete s;
    }
};

using unique_ifstream = std::unique_ptr<std::ifstream, StreamDel<std::ifstream>>;

unique_ifstream OpenFile(const char* path) {
    return unique_ifstream{new std::ifstream{path, std::ios::binary}};
}

template <bool Swap, typename... Args>
std::tuple<Args...> unpack(unique_ifstream& stream) {
    size_t size = GetTypesSize<Args...>();
    u8* ptr = new u8[size];
    stream->read((char*)ptr, size);
    std::tuple<Args...> result = unpack<Swap, Args...>(ptr);
    delete [] ptr;
    return result;
}