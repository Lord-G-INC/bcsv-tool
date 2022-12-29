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
}

void Writer::GenerateFields() {
    std::string s = text.substr(0, text.find('\n'));
    text = text.substr(text.find('\n')+1);
    std::vector<std::string> names{};
    size_t pos = 0;
    std::string token;
    while ((pos = s.find(',')) != text.npos) {
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
        if (n.find('x') == 1) {
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
    for (int i = 0; i < fields.size(); i++) {
        auto& f = fields[i];
        f.dataoff = doff;
        doff += BCSV::GetDTSize(f);
    }
    return;
}