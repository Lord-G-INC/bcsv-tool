#include "unpack.cc"
#include "reader.cc"
#include "writer.cc"
#if __cplusplus < 201703L
#include "filesystem.hpp"
using namespace ghc;
#else
#include <filesystem>
using namespace std;
#endif

void HandleBCSV(const filesystem::path& arg, const filesystem::path& dir) {
    Reader reader{arg.string().c_str(), (dir / "lookup_supermariogalaxy.txt").string().c_str()};
    filesystem::path out{arg};
    reader.WriteCSV(out.replace_extension(".csv").string().c_str());
}

void HandleCSV(const filesystem::path& arg) {
    Writer writer{arg.string().c_str()};
    writer.GenerateFields();
    auto vals = writer.GetValues();
    auto table = writer.CreateStringTable(vals);
    auto buffer = writer.CreateBuffer(table);
    auto doff = writer.FillHeaderAndFields(buffer);
    writer.FillFieldTable(buffer, doff, vals);
    filesystem::path out{arg};
    writer.WriteBCSV(out.replace_extension(".bcsv").string().c_str(), buffer);
}

int main(int argc, char* argv[]) {
    const filesystem::path exe = argv[0];
    const auto dir = exe.parent_path();
    std::vector<std::string> args{(size_t)argc - 1};
    std::copy(argv + 1, argv + 1 + argc - 1, args.begin());
    for (auto arg : args) {
        filesystem::path p{arg};
        auto ext = p.extension().string();
        if (ext == ".bcsv" || ext == ".tbl" || ext == ".banmt" || ext == "") {
            HandleBCSV(arg, dir);
        } else if (ext == ".csv") {
            HandleCSV(arg);
        }
    }
    return 0;
}