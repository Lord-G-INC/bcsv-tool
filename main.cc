#include "unpack.cc"
#include "bcsv.hh"

int main() {
    auto hashes = ReadHashes("lookup_supermariogalaxy.txt");
    printf("%d", hashes.size());
}