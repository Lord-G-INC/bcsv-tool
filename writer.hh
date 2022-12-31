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
    std::vector<u8> CreateBuffer(std::string& table) {
        size_t stringoff = header.entrydataoff+header.entrysize*header.entrycount;
        size_t size = stringoff + table.size();
        size += ((size + 31 & ~31) - size);
        std::vector<u8> result{};
        for (int i = 0; i < size; i++)
            result.push_back(0x40);
        std::copy(table.begin(), table.end(), result.begin() + stringoff);
        return result;
    }
    size_t FillHeaderAndFields(std::vector<u8>&);
    void FillFieldTable(std::vector<u8>&, size_t, std::vector<std::vector<std::string>>&);
    void WriteBCSV(const char*, std::vector<u8>&);
};