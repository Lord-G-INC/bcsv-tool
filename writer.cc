#include "writer.hh"
#include <string.h>

Writer::Writer(const char* csv) : Writer() {
    auto stream = OpenReader(csv);
    stream->seekg(0, stream->end);
    size_t size = stream->tellg();
    stream->seekg(0, stream->beg);
    std::unique_ptr<char[]> data{new char[size]};
    memset(data.get(), 0, size);
    stream->read(data.get(), size);
    text = std::string{data.get(), size};
    for (auto pos = text.find('\r'); pos != text.npos; pos = text.find('\r', pos)) {
        text.erase(pos, 1);
    }
}

void Writer::GenerateFields() {
    std::string s = text.substr(0, text.find('\n'));
    text = text.substr(text.find('\n')+1);
    std::vector<std::string> names{};
    std::string token;
    for (size_t pos = s.find(','); pos != text.npos; pos = s.find(',')) {
        token = s.substr(0, pos);
        names.push_back(token);
        s.erase(0, pos + 1);
    }
    names.push_back(s);
    s.erase(0);
    fields.reserve(names.size());
    for (auto& name : names) {
        BCSV::Field field{};
        std::string type = name.substr(name.find(':')+1);
        std::string n = name.substr(0, name.find(':'));
        if (n.find('x') == 1 && n.find('0') == 0) {
            u32 num = std::stoul(n, nullptr, 16);
            field.hash = num;
        } else {
            field.hash = CalcHash(n.c_str());
        }
        u8 t = std::stoul(type);
        field.type = t;
        fields.push_back(field);
    }
    s16 doff = 0;
    std::vector<BCSV::Field> clone{fields};
    std::sort(clone.begin(), clone.end(), [](BCSV::Field& a, BCSV::Field& b){return a.type < b.type;});
    for (auto& f : clone) {
        f.dataoff = doff;
        doff += BCSV::GetDTSize(f);
        auto& og = *std::find_if(fields.begin(), fields.end(), [&](BCSV::Field& x){return x.hash == f.hash;});
        og.dataoff = f.dataoff;
    }
    header.fieldcount = fields.size();
    header.entrysize = doff;
    header.entrydataoff = sizeof(header) + sizeof(BCSV::Field) * header.fieldcount;
    return;
}

std::vector<std::vector<std::string>> Writer::GetValues() {
    std::vector<std::vector<std::string>> result{};
    for (size_t pos = text.find('\n'); pos != text.npos; pos = text.find('\n')) {
        std::vector<std::string> vec;
        auto line = text.substr(0, pos);
        for (size_t lpos = line.find(','); lpos != line.npos; lpos = line.find(',')) {
            vec.push_back(line.substr(0, lpos));
            line.erase(0, lpos+1);
        }
        vec.push_back(line);
        result.push_back(vec);
        text = text.substr(pos+1);
    }
    if (!text.empty()) {
        std::vector<std::string> vec;
        auto line = text;
        for (size_t lpos = line.find(','); lpos != line.npos; lpos = line.find(',')) {
            vec.push_back(line.substr(0, lpos));
            line.erase(0, lpos+1);
        }
        vec.push_back(line);
        result.push_back(vec);
    }
    header.entrycount = result.size();
    return result;
}

std::string Writer::CreateStringTable(std::vector<std::vector<std::string>>& vals) {
    std::string result{};
    std::map<std::string, size_t> offs{};
    size_t off = 0;
    for (int row = 0; row < header.entrycount; row++) {
        for (int i = 0; i < header.fieldcount; i++) {
            auto& f = fields[i];
            if (f.type == 6) {
                auto& str = vals[row][i];
                auto val = str + '\0';
                if (offs.count(val) == 0) {
                    result += val;
                    offs[val] = off;
                    off += str.size() + 1;
                }
                vals[row][i] = string_format("%d", offs[val]);
            }
        }
    }
    return result;
}

namespace {
    template <typename T, bool Swap = true>
    std::array<u8, sizeof(T)> GetBytes(T val) {
        if (Swap && sizeof(T) > 1)
            SwapVal(val);
        union U
        {
            std::array<u8, sizeof(T)> res;
            T raw;
        } buf;
        buf.raw = val;
        return buf.res;
    }
    template <typename T, bool Swap = true>
    void write(T val, std::vector<u8>& vec, size_t& off) {
        std::array<u8, sizeof(T)> buffer = GetBytes<T, Swap>(val);
        std::copy(buffer.begin(), buffer.end(), vec.begin() + off);
        off += sizeof(T);
    }
}

size_t Writer::FillHeaderAndFields(std::vector<u8>& vec) {
    size_t off = 0;
    write(header.entrycount, vec, off);
    write(header.fieldcount, vec, off);
    write(header.entrydataoff, vec, off);
    write(header.entrysize, vec, off);
    for (int i = 0; i < fields.size(); i++) {
        auto& f = fields[i];
        write(f.hash, vec, off);
        write(f.mask, vec, off);
        write(f.dataoff, vec, off);
        write(f.shift, vec, off);
        write(f.type, vec, off);
    }
    return off;
}

void Writer::FillFieldTable(std::vector<u8>& vec, size_t dataoff, std::vector<std::vector<std::string>>& vals) {
    for (int row = 0; row < header.entrycount; row++) {
        for (int i = 0; i < fields.size(); i++) {
            auto& f = fields[i];
            auto& str = vals[row][i];
            size_t off = dataoff + row * header.entrysize + f.dataoff;
            switch (f.type) {
                default:{break;}
                case BCSV::DataType::LONG:
                case BCSV::DataType::LONG_2:
                case BCSV::DataType::STRING_OFF: {
                    u32 num = std::stoul(str);
                    write(num, vec, off);
                    break;
                }
                case BCSV::DataType::SHORT: {
                    u16 num = std::stoul(str);
                    write(num, vec, off);
                    break;
                }
                case BCSV::DataType::FLOAT: {
                    f32 num = std::stof(str);
                    write(num, vec, off);
                    break;
                }
                case BCSV::DataType::CHAR: {
                    s8 num = std::stol(str);
                    write(num, vec, off);
                    break;
                }
            }
        }
    }
}

void Writer::WriteBCSV(const char* path, std::vector<u8>& vec) {
    auto stream = OpenWriter(path);
    stream->write((char*)vec.data(), vec.size());
}