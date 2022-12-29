#include "unpack.cc"
#include "reader.cc"

int main() {
    Reader reader{"ActionSound.bcsv", "lookup_supermariogalaxy.txt"};
    reader.WriteCSV("test.csv");
}