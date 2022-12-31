#include "unpack.cc"
#include "reader.cc"
#include "writer.cc"

int main() {
    Writer writer{"test.csv"};
    writer.GenerateFields();
    auto vals = writer.GetValues();
    auto table = writer.CreateStringTable(vals);
    auto buffer = writer.CreateBuffer(table);
    auto doff = writer.FillHeaderAndFields(buffer);
    writer.FillFieldTable(buffer, doff, vals);
    writer.WriteBCSV("test.bcsv", buffer);
    return 0;
}