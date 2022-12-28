#include "unpack.cc"
#include "bcsv.hh"

int main() {
    auto sedt = OpenFile("StageEventDataTable.bcsv");
    BCSV::Header head = BCSV::ReadHeader(sedt);
    auto fields = BCSV::ReadFields(sedt, head);
    auto dict = ReadHashes("lookup_supermariogalaxy.txt");
    auto names = BCSV::GetFeildNames(fields, dict);
    printf("%d", names.size());
}