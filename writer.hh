#pragma once

#include "bcsv.hh"

struct Writer {
    std::string text;
    BCSV::Header header;
    std::vector<BCSV::Field> fields;
    Writer() : header(), fields(), text() {}
    Writer(const char*);
    void GenerateFields();
};