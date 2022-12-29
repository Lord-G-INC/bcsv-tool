#include "reader.hh"

Reader::Reader(const char* bcsv, const char* hash) {
    stream = OpenReader(bcsv);
    header = BCSV::ReadHeader(stream);
    Fields = BCSV::ReadFields(stream, header);
    Hashes = ReadHashes(hash);
}

void Reader::WriteCSV(const char* path) {
    auto writer = OpenWriter(path);
    std::string text{};
    std::vector<std::string> names{};
    for (auto& f : Fields) {
        u32& h = f.hash;
        if (Hashes.count(h) == 1) {
            names.push_back(Hashes[h]);
        } else {
            names.push_back(string_format("0x%x", h));
        }
    }
    for (auto i = 0; i < names.size(); i++) {
        bool last = i == names.size() - 1;
        text += names[i];
        text += !last ? ',' : '\n';
    }
    for (auto row = 0; row < header.entrycount; row++) {
        for (auto i = 0; i < Fields.size(); i++) {
            bool last = i == Fields.size() - 1;
            BCSV::Field& f = Fields[i];
            std::string fmt{};
            fmt += BCSV::GetDTFmt(f);
            fmt += !last ? ',' : '\n';
            switch (f.type) {
                case BCSV::DataType::LONG:
                case BCSV::DataType::LONG_2: { 
                    s32 val = BCSV::ReadType<s32>(stream, header, row, f);
                    text += string_format(fmt, val);
                    break; 
                }
                case BCSV::DataType::CHAR: {
                    char val = BCSV::ReadType<char>(stream, header, row, f);
                    text += string_format(fmt, val);
                    break;
                }
                case BCSV::DataType::SHORT: {
                    s16 val = BCSV::ReadType<s16>(stream, header, row, f);
                    text += string_format(fmt, val);
                    break;
                }
                case BCSV::DataType::FLOAT: {
                    f32 val = BCSV::ReadType<f32>(stream, header, row, f);
                    text += string_format(fmt, val);
                    break;
                }
                case BCSV::DataType::STRING_OFF: {
                    auto str = BCSV::ReadStringOff(stream, header, row, f);
                    text += string_format(fmt, str.c_str());
                    break;
                }
                default: { break; }
            }
        }
    }
    writer->write(text.c_str(), text.size());
}