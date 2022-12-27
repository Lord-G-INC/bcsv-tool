#include "unpack.cc"
#include "bcsv.hh"
#include <iostream>
#include <fstream>
#include <iostream>

u8* ReadFile(const char* path) {
    std::streampos size;
    std::ifstream inFile;
    u8* returnchar;

	inFile.open(path, std::ios::in | std::ios::binary | std::ios::ate);

    if (inFile.is_open()) {
        size = inFile.tellg();
        inFile.seekg(0, std::ios::beg);
        returnchar = new u8[size];
        inFile.read((char*)returnchar, size);
		inFile.close();
    }
    return returnchar;
}

int main() {
    u8* sedt = ReadFile("WarpAreaStageTable.bcsv");
    BCSV::Header head = BCSV::ReadHeader(sedt);
    sedt += sizeof(BCSV::Header);
    printf("entries: %d, fields: %d, dataoff: %d, entrysize: %d\n", 
    head.entrycount, head.fieldcount, head.entrydataoff, head.entrysize);
    BCSV::Field* fields = BCSV::ReadFields(sedt, head);
    printf("0x%x", fields[0].hash);
}