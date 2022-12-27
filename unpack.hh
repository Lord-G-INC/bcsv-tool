#pragma once

#include <tuple>
#include <stddef.h>
#include <array>
#include <algorithm>
#include <iostream>


template <bool Swap, typename... Args>
std::tuple<Args...> unpack(unsigned char* data);

template <typename... Args>
void unpacktuple(std::tuple<Args...>&, Args*...);