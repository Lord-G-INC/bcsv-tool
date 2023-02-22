#include "reader.hh"

Reader::Reader(const char* bcsv, const char* hash) {
    stream = OpenReader(bcsv);
    header = BCSV::ReadHeader(stream);
    fields = BCSV::ReadFields(stream, header);
    hashes = ReadHashes(hash);
}

void Reader::WriteCSV(const char* path) {
    auto writer = OpenWriter(path);
    std::string text{};
    std::vector<std::string> names{};
    for (auto& f : fields) {
        u32& h = f.hash;
        if (hashes.count(h) == 1) {
            names.push_back(hashes[h]);
        } else {
            names.push_back(string_format("0x%x", h));
        }
    }
    for (auto i = 0; i < names.size(); i++) {
        bool last = i == names.size() - 1;
        text += names[i] + ':' + string_format("%d", fields[i].type);
        text += !last ? ',' : '\n';
    }
    for (auto row = 0; row < header.entrycount; row++) {
        for (auto i = 0; i < fields.size(); i++) {
            bool last = i == fields.size() - 1;
            BCSV::Field& f = fields[i];
            std::string fmt{};
            fmt += BCSV::GetDTFmt(f);
            fmt += !last ? ',' : '\n';
            switch (f.type) {
                case BCSV::DataType::LONG:
                case BCSV::DataType::LONG_2: { 
                    u32 val = BCSV::ReadType<u32>(stream, header, row, f);
                    val = (val & f.mask) >> f.shift;
                    val |= (val & 0x80000000) == 0x80000000 ? ~0xFFFFFFFF : val;
                    text += string_format(fmt, val);
                    break; 
                }
                case BCSV::DataType::CHAR: {
                    int val = BCSV::ReadType<char>(stream, header, row, f) & f.mask;
                    val >>= f.shift;
                    val |= (val & 0x80) == 0x80 ? ~0xFF : val;
                    text += string_format(fmt, val);
                    break;
                }
                case BCSV::DataType::SHORT: {
                    u16 val = BCSV::ReadType<u16>(stream, header, row, f);
                    val = (val & f.mask) >> f.shift;
                    val |= (val & 0x8000) == 0x8000 ? ~0xFFFF : val;
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

