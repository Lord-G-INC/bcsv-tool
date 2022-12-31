#pragma once

#include "bcsv.hh"

struct Writer {
    private:
    std::string text;
    public:
    BCSV::Header header;
    std::vector<BCSV::Field> fields;
    Writer() : header(), fields(), text() {}
    Writer(const char*);
    void GenerateFields();
    std::vector<std::vector<std::string>> GetValues();
    std::string CreateStringTable(std::vector<std::vector<std::string>>&);
    void WriteBCSV(const char*, std::vector<std::vector<std::string>>&, std::string);
};