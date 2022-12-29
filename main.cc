#include "unpack.cc"
#include "reader.cc"
#include "writer.cc"

int main() {
    Writer writer{"test.csv"};
    writer.GenerateFields();
}