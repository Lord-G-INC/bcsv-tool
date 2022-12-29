#pragma once

#include "bcsv.hh"

struct Reader {
    unique_ifstream stream;
    BCSV::Header header;
    std::vector<BCSV::Field> fields;
    std::map<u32, std::string> hashes;
    Reader(const char*, const char*);
    void WriteCSV(const char*);
};