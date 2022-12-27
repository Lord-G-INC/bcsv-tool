#include "unpack.cc"
#include "bcsv.hh"

int main() {
    auto sedt = OpenFile("StageEventDataTable.bcsv");
    BCSV::Header head = BCSV::ReadHeader(sedt);
    auto fields = BCSV::ReadFields(sedt, head);
    auto first = fields[2];
    auto val = BCSV::ReadStringOff(sedt, head, 16, first);
    printf("%s", val.c_str());
}