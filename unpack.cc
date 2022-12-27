#include "unpack.hh"


namespace {
    template <typename T>
    using arr = std::array<unsigned char, sizeof(T)>;

    template <typename T>
    T read(unsigned char* data) {
        union U
        {
            T raw;
            arr<T> buf;
        } src;
        std::reverse_copy(data, data + sizeof(T), src.buf.begin());
        return src.raw;
    }

    template <size_t I, typename... Ts>
    void unpack(unsigned char* data, std::tuple<Ts...>& tup) {
        using result = std::tuple_element_t<I, std::tuple<Ts...>>;
        result r = read<result>(data);
        std::get<I>(tup) = r;
        return;
    }    

    template <bool Swap, size_t I, typename... Ts>
    typename std::enable_if<I == sizeof...(Ts)>::type unpack(unsigned char* data, std::tuple<Ts...>& tup) {
        return;
    }

    template <bool Swap, size_t I, typename... Ts>
    typename std::enable_if<I < sizeof...(Ts)>::type unpack(unsigned char* data, std::tuple<Ts...>& tup) {
        using result = std::tuple_element_t<I, std::tuple<Ts...>>;
        unpack<I, Ts...>(data, tup);
        if (Swap && sizeof(result) > 1)
            SwapVal(std::get<I>(tup));
        unpack<Swap, I+1, Ts...>(data + sizeof(result), tup);
        return;
    }
}

template<bool Swap, typename... Args>
std::tuple<Args...> unpack(unsigned char* data) {
    std::tuple<Args...> result{};
    unpack<Swap, 0, Args...>(data, result);
    return result;
}

namespace {
    template <size_t I, typename... Args>
    typename std::enable_if<I == sizeof...(Args)>::type 
    unpacktuple(std::tuple<Args...>& tup, std::tuple<Args*...>& vals) {
        return;
    }
    template <size_t I, typename... Args>
    typename std::enable_if<I < sizeof...(Args)>::type 
    unpacktuple(std::tuple<Args...>& tup, std::tuple<Args*...>& vals) {
        using result = std::tuple_element_t<I, std::tuple<Args...>>;
        result& src = std::get<I>(tup);
        result* dst = std::get<I>(vals);
        *dst = src;
        unpacktuple<I+1, Args...>(tup, vals);
        return;
    }
}

template <typename... Args>
void unpacktuple(std::tuple<Args...>& tup, Args*... args) {
    std::tuple<Args*...> vals = std::make_tuple(args...);
    unpacktuple<0, Args...>(tup, vals);
    return;
}